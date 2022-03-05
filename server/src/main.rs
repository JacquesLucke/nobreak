use anyhow::anyhow;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, serde::Serialize)]
struct FailInfo {
    key: Key,
    message: String,
}

impl std::fmt::Display for FailInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{:?}: {:?}>", self.key, self.message)
    }
}

struct NobreakState {
    cli_args: CliArgs,
    directory: std::path::PathBuf,
    fails: std::sync::Arc<std::sync::Mutex<Vec<FailInfo>>>,
}

#[rocket::post("/_shutdown")]
fn handle_shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutdown"
}

#[derive(Clone, Debug, serde::Serialize)]
struct Key {
    sub_keys: Vec<String>,
}

#[derive(Debug)]
struct LoadRequest {
    key: Key,
}

#[derive(Debug)]
struct LogValueRequest {
    key: Key,
    value: Vec<u8>,
}

#[derive(Debug)]
struct LogSuccessRequest {
    key: Key,
}

#[derive(Debug)]
struct LogFailRequest {
    key: Key,
    message: String,
}

#[derive(Debug)]
struct StatusRequest {}

#[derive(Debug)]
enum RequestType {
    Load(LoadRequest),
    LogValue(LogValueRequest),
    LogSuccess(LogSuccessRequest),
    LogFail(LogFailRequest),
    Status(StatusRequest),
}

#[derive(Debug)]
struct RequestMessage {
    version: u32,
    content: RequestType,
}

fn decode_request(request_buffer: &[u8]) -> anyhow::Result<RequestMessage> {
    let mut cursor = std::io::Cursor::new(request_buffer);
    let version = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor)?;
    let opcode = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor)?;
    let content = match opcode {
        0 => RequestType::Status(StatusRequest {}),
        1 => {
            let key = decode_key(&mut cursor)?;
            RequestType::Load(LoadRequest { key })
        }
        2 => {
            let key = decode_key(&mut cursor)?;
            let value = decode_bytes(&mut cursor)?;
            RequestType::LogValue(LogValueRequest { key, value })
        }
        3 => {
            let key = decode_key(&mut cursor)?;
            RequestType::LogSuccess(LogSuccessRequest { key })
        }
        4 => {
            let key = decode_key(&mut cursor)?;
            let message = decode_str(&mut cursor)?;
            RequestType::LogFail(LogFailRequest { key, message })
        }
        _ => return Err(anyhow!("Invalid opcode")),
    };
    Ok(RequestMessage { version, content })
}

fn decode_key(mut cursor: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Key> {
    let mut key = Key { sub_keys: vec![] };
    let amount = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor)?;
    for _ in 0..amount {
        key.sub_keys.push(decode_str(&mut cursor)?);
    }
    Ok(key)
}

fn decode_bytes(mut cursor: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Vec<u8>> {
    let size = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor)?;
    let mut buffer = vec![0; size as usize];
    std::io::Read::read_exact(&mut cursor, &mut buffer)?;
    Ok(buffer)
}

fn decode_str(mut cursor: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<String> {
    let bytes = decode_bytes(&mut cursor)?;
    Ok(String::from_utf8(bytes)?)
}

fn answer_status_request() -> Vec<u8> {
    vec![b'C']
}

fn key_to_path(key: &Key, state: &NobreakState) -> std::path::PathBuf {
    let mut path = state.directory.clone();
    for sub_key in key.sub_keys.iter() {
        path.push(sub_key);
    }
    path.set_extension("txt");
    path
}

async fn answer_load_request(request: LoadRequest, state: &NobreakState) -> Vec<u8> {
    match handle_load_request(request, state).await {
        Ok(value) => value,
        Err(_) => vec![0],
    }
}

async fn handle_load_request(
    request: LoadRequest,
    state: &NobreakState,
) -> anyhow::Result<Vec<u8>> {
    let path = key_to_path(&request.key, state);
    let mut file = tokio::fs::File::open(path).await?;
    let mut contents = vec![1];
    file.read_buf(&mut contents).await?;
    Ok(contents)
}

async fn answer_log_value_request(request: LogValueRequest, state: &NobreakState) -> Vec<u8> {
    match handle_log_value_request(request, state).await {
        Ok(_) => vec![b'E'],
        Err(_) => vec![b'Z'],
    }
}

async fn handle_log_value_request(
    request: LogValueRequest,
    state: &NobreakState,
) -> anyhow::Result<()> {
    let path = key_to_path(&request.key, state);
    let mut builder = tokio::fs::DirBuilder::new();
    builder.recursive(true);
    let parent_path = path.parent().ok_or(anyhow!("error creating directory"))?;
    builder.create(parent_path).await?;
    tokio::fs::File::create(&path)
        .await?
        .write_all(&request.value)
        .await?;
    Ok(())
}

async fn answer_log_success_request(request: LogSuccessRequest) -> Vec<u8> {
    vec![b'F']
}

async fn answer_log_fail_request(
    request: LogFailRequest,
    state: &rocket::State<NobreakState>,
) -> Vec<u8> {
    match handle_log_fail_request(&request, state).await {
        Ok(_) => vec![],
        Err(_) => vec![],
    }
}

async fn handle_log_fail_request(
    request: &LogFailRequest,
    state: &rocket::State<NobreakState>,
) -> anyhow::Result<()> {
    state.fails.lock().unwrap().push(FailInfo {
        key: request.key.clone(),
        message: request.message.clone(),
    });
    Ok(())
}

#[rocket::get("/api", data = "<msg>")]
async fn handle_api(msg: &[u8], state: &rocket::State<NobreakState>) -> Vec<u8> {
    let request = match decode_request(msg) {
        Ok(request) => request,
        _ => return vec![b'A'],
    };
    println!("{:?}", request);
    match request.content {
        RequestType::Status(_) => answer_status_request(),
        RequestType::Load(request) => answer_load_request(request, state).await,
        RequestType::LogValue(request) => answer_log_value_request(request, state).await,
        RequestType::LogSuccess(request) => answer_log_success_request(request).await,
        RequestType::LogFail(request) => answer_log_fail_request(request, state).await,
    }
}

#[rocket::get("/")]
fn handle_index() -> &'static str {
    "Nobreak server is active."
}

#[rocket::get("/server_id")]
fn handle_server_id(state: &rocket::State<NobreakState>) -> String {
    state.cli_args.server_id.to_string()
}

#[rocket::get("/log")]
fn handle_log(state: &rocket::State<NobreakState>) -> String {
    let fails: &Vec<FailInfo> = &state.fails.lock().unwrap();
    serde_json::to_string(&fails).unwrap()
}

#[derive(Parser, Debug)]
#[clap(version)]
struct CliArgs {
    #[clap(long)]
    directory: Option<std::path::PathBuf>,

    #[clap(long)]
    #[clap(default_value_t = std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)))]
    address: std::net::IpAddr,

    #[clap(long)]
    #[clap(default_value_t = 2345)]
    port: u16,

    #[clap(long)]
    #[clap(default_value_t = 0)]
    server_id: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cli_args = CliArgs::parse();

    if cli_args.directory.is_none() {
        let default_directory = std::env::current_dir()?.join("nobreak_data");
        cli_args.directory = Some(default_directory);
    }

    println!("{:?}", cli_args);

    let fails = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    let config = rocket::Config {
        port: cli_args.port,
        address: cli_args.address,
        ..Default::default()
    };

    rocket::build()
        .configure(config)
        .manage(NobreakState {
            cli_args: cli_args,
            directory: std::path::Path::new("/home/jacques/Documents/nobreak/testing").to_owned(),
            fails: fails.clone(),
        })
        .mount(
            "/",
            rocket::routes![
                handle_shutdown,
                handle_api,
                handle_index,
                handle_server_id,
                handle_log
            ],
        )
        .launch()
        .await?;

    for fail_info in fails.lock().unwrap().iter() {
        println!("Failed: {}", fail_info);
    }
    println!("Finished nobreak.");
    Ok(())
}
