use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

#[derive(serde::Serialize, Clone, Copy)]
enum OperationMode {
    Update,
    Check,
}

#[derive(Clone, Debug)]
struct FailInfo {
    key: String,
    msg: String,
}

impl std::fmt::Display for FailInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<{}: {}>", self.key, self.msg)
    }
}

struct NobreakState {
    mode: OperationMode,
    directory: std::path::PathBuf,
    fails: std::sync::Arc<std::sync::Mutex<Vec<FailInfo>>>,
}

#[rocket::post("/_shutdown")]
fn handle_shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutdown"
}

#[derive(Debug)]
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
        _ => return Err(anyhow::anyhow!("Invalid opcode")),
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
        Err(_) => vec![],
    }
}

async fn handle_load_request(
    request: LoadRequest,
    state: &NobreakState,
) -> anyhow::Result<Vec<u8>> {
    let path = key_to_path(&request.key, state);
    let mut file = tokio::fs::File::open(path).await?;
    let mut contents = vec![];
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
    let parent_path = path
        .parent()
        .ok_or(anyhow::anyhow!("error creating directory"))?;
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

async fn answer_log_fail_request(request: LogFailRequest) -> Vec<u8> {
    vec![b'G']
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
        RequestType::LogFail(request) => answer_log_fail_request(request).await,
    }
}

#[tokio::main]
async fn execute_script(
    script_path: &std::path::Path,
    server_url: reqwest::Url,
) -> anyhow::Result<()> {
    tokio::process::Command::new("sh")
        .arg(script_path)
        .env("NOBREAK_SERVER_URL", server_url.to_string())
        .spawn()
        .expect("failed to spawn")
        .wait()
        .await?;
    let client = reqwest::Client::new();
    let shutdown_url = server_url.join("_shutdown")?;
    client.post(shutdown_url).send().await?;
    Ok(())
}

fn on_liftoff(script_path: std::path::PathBuf, rocket: &rocket::Rocket<rocket::Orbit>) {
    let address = rocket.config().address;
    let port = rocket.config().port;
    let mut server_url =
        reqwest::Url::parse(&("http://".to_owned() + &address.to_string())).unwrap();
    server_url.set_port(Some(port)).unwrap();
    std::thread::spawn(move || -> anyhow::Result<()> {
        execute_script(&script_path, server_url)?;
        Ok(())
    });
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let full_key: Vec<String> = vec!["test", "a", "b", "c"]
    //     .iter()
    //     .map(|&s| s.to_owned())
    //     .collect();
    // let encoded_full_key = encode_full_key(&full_key);
    // for b in encoded_full_key.iter() {
    //     print!("{} ", b);
    // }
    // println!();
    // let decoded_full_key = decode_full_key(&encoded_full_key);
    // for key in decoded_full_key.iter() {
    //     print!("{}, ", key);
    // }
    // println!();

    // let matches = clap::App::new("nobreak")
    //     .arg(
    //         clap::Arg::with_name("directory")
    //             .help("Directory where the recorded data is stored.")
    //             .required(true)
    //             .index(1),
    //     )
    //     .arg(
    //         clap::Arg::with_name("script")
    //             .help("Shell script that runs the regression test suite")
    //             .required(true)
    //             .index(2),
    //     )
    //     .subcommand(clap::SubCommand::with_name("check"))
    //     .subcommand(clap::SubCommand::with_name("update"))
    //     .get_matches();

    // let operation_mode = match matches.subcommand_name() {
    //     None => {
    //         println!("Use a subcommand");
    //         return Ok(());
    //     }
    //     Some(name) => match name {
    //         "check" => OperationMode::Check,
    //         "update" => OperationMode::Update,
    //         _ => panic!(),
    //     },
    // };

    // let directory_path = std::path::Path::new(matches.value_of("directory").unwrap()).to_owned();
    // let script_path = std::path::Path::new(matches.value_of("script").unwrap()).to_owned();

    // if !directory_path.exists() || !directory_path.is_dir() {
    //     println!("Directory does not exist: {}", &directory_path.display());
    //     return Ok(());
    // }

    let fails = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    rocket::build()
        .manage(NobreakState {
            mode: OperationMode::Check,
            directory: std::path::Path::new("/home/jacques/Documents/nobreak/testing").to_owned(),
            fails: fails.clone(),
        })
        .mount("/", rocket::routes![handle_shutdown, handle_api,])
        // .attach(rocket::fairing::AdHoc::on_liftoff(
        //     "Start Script",
        //     move |rocket| {
        //         Box::pin(async move {
        //             on_liftoff(script_path, rocket);
        //         })
        //     },
        // ))
        .launch()
        .await?;

    for fail_info in fails.lock().unwrap().iter() {
        println!("Failed: {}", fail_info);
    }
    println!("Finished nobreak.");
    Ok(())
}
