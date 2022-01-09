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

#[derive(serde::Serialize)]
struct IndexResponseMessage {
    mode: OperationMode,
    log: &'static str,
    get: &'static str,
    fail: &'static str,
}

#[rocket::get("/")]
fn handle_index(state: &rocket::State<NobreakState>) -> String {
    let res = IndexResponseMessage {
        mode: state.mode,
        log: "/log/",
        get: "/get/",
        fail: "/fail/",
    };
    serde_json::to_string(&res).unwrap()
}

#[rocket::post("/log/<key>", data = "<msg>")]
async fn handle_log(key: &str, msg: &[u8], state: &rocket::State<NobreakState>) -> String {
    match store_value_for_key(key, msg, state).await {
        Ok(_) => "Success.".to_owned(),
        Err(_) => "Failed.".to_owned(),
    }
}

#[rocket::get("/get/<key>")]
async fn handle_get(key: &str, state: &rocket::State<NobreakState>) -> Vec<u8> {
    match load_value_for_key(key, state).await {
        Ok(vec) => vec,
        Err(_) => vec![],
    }
}

#[rocket::post("/fail/<key>", data = "<msg>")]
async fn handle_fail(key: &str, msg: &str, state: &rocket::State<NobreakState>) -> &'static str {
    state.fails.lock().unwrap().push(FailInfo {
        key: key.to_owned(),
        msg: msg.to_owned(),
    });
    "."
}

#[rocket::post("/_shutdown")]
fn handle_shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutdown"
}

fn get_path_for_key(key: &str, state: &NobreakState) -> std::path::PathBuf {
    state.directory.join(key).with_extension("txt")
}

async fn load_value_for_key(key: &str, state: &NobreakState) -> anyhow::Result<Vec<u8>> {
    let path = get_path_for_key(key, state);
    let mut file = tokio::fs::File::open(path).await?;
    let mut contents = vec![];
    file.read_buf(&mut contents).await?;
    Ok(contents)
}

async fn store_value_for_key(key: &str, value: &[u8], state: &NobreakState) -> anyhow::Result<()> {
    let path = get_path_for_key(key, state);
    tokio::fs::File::create(path)
        .await?
        .write_all(value)
        .await?;
    Ok(())
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

fn encode_full_key(full_key: &[String]) -> Vec<u8> {
    let mut buffer = vec![];
    WriteBytesExt::write_u32::<NetworkEndian>(&mut buffer, full_key.len() as u32).unwrap();
    for key in full_key {
        WriteBytesExt::write_u32::<NetworkEndian>(&mut buffer, key.len() as u32).unwrap();
        buffer.extend(key.as_bytes());
    }
    buffer
}

fn decode_full_key(buffer: &[u8]) -> Vec<String> {
    let mut full_key = vec![];
    let mut cursor = std::io::Cursor::new(buffer);
    let key_amount = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor).unwrap();
    for _ in 0..key_amount {
        let key_length = ReadBytesExt::read_u32::<NetworkEndian>(&mut cursor).unwrap();
        let mut key_bytes: Vec<u8> = vec![0; key_length as usize];
        std::io::Read::read_exact(&mut cursor, &mut key_bytes).unwrap();
        let key = String::from_utf8(key_bytes).unwrap();
        full_key.push(key);
    }
    return full_key;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let full_key: Vec<String> = vec!["test", "a", "b", "c"]
        .iter()
        .map(|&s| s.to_owned())
        .collect();
    let encoded_full_key = encode_full_key(&full_key);
    for b in encoded_full_key.iter() {
        print!("{} ", b);
    }
    println!();
    let decoded_full_key = decode_full_key(&encoded_full_key);
    for key in decoded_full_key.iter() {
        print!("{}, ", key);
    }
    println!();

    let matches = clap::App::new("nobreak")
        .arg(
            clap::Arg::with_name("directory")
                .help("Directory where the recorded data is stored.")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("script")
                .help("Shell script that runs the regression test suite")
                .required(true)
                .index(2),
        )
        .subcommand(clap::SubCommand::with_name("check"))
        .subcommand(clap::SubCommand::with_name("update"))
        .get_matches();

    let operation_mode = match matches.subcommand_name() {
        None => {
            println!("Use a subcommand");
            return Ok(());
        }
        Some(name) => match name {
            "check" => OperationMode::Check,
            "update" => OperationMode::Update,
            _ => panic!(),
        },
    };

    let directory_path = std::path::Path::new(matches.value_of("directory").unwrap()).to_owned();
    let script_path = std::path::Path::new(matches.value_of("script").unwrap()).to_owned();

    if !directory_path.exists() || !directory_path.is_dir() {
        println!("Directory does not exist: {}", &directory_path.display());
        return Ok(());
    }

    let fails = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

    rocket::build()
        .manage(NobreakState {
            mode: operation_mode,
            directory: directory_path,
            fails: fails.clone(),
        })
        .mount(
            "/",
            rocket::routes![
                handle_index,
                handle_log,
                handle_get,
                handle_fail,
                handle_shutdown
            ],
        )
        .attach(rocket::fairing::AdHoc::on_liftoff(
            "Start Script",
            move |rocket| {
                Box::pin(async move {
                    on_liftoff(script_path, rocket);
                })
            },
        ))
        .launch()
        .await?;

    for fail_info in fails.lock().unwrap().iter() {
        println!("Failed: {}", fail_info);
    }
    println!("Finished nobreak.");
    Ok(())
}
