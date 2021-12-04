use tokio::io::AsyncWriteExt;

#[derive(serde::Serialize)]
enum OperationMode {
    Update,
    // Check,
}

struct NobreakState {
    directory: std::path::PathBuf,
}

#[derive(serde::Serialize)]
struct IndexResponseMessage {
    mode: OperationMode,
    log: &'static str,
    get: &'static str,
}

#[rocket::get("/")]
fn handle_index() -> String {
    let res = IndexResponseMessage {
        mode: OperationMode::Update,
        log: "/log/",
        get: "/get/",
    };
    serde_json::to_string(&res).unwrap()
}

#[rocket::post("/log/<key>", data = "<msg>")]
async fn handle_log(key: &str, msg: &[u8], state: &rocket::State<NobreakState>) -> String {
    println!("Key: {}", &key);
    for v in msg {
        println!("Value: {}", &v);
    }
    let path = state.directory.join(key).with_extension("txt");
    let mut file = match tokio::fs::File::create(path).await {
        Ok(file) => file,
        Err(_) => return "Could not create file.".to_owned(),
    };
    match file.write_all(msg).await {
        Ok(_) => return "Success.".to_owned(),
        Err(_) => return "Could not write to file.".to_owned(),
    }
}

#[rocket::get("/get/<_key>")]
fn handle_get(_key: &str) -> &'static [u8] {
    &[70, 71, 72, 73]
}

#[rocket::post("/_shutdown")]
fn handle_shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutdown"
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
        .get_matches();
    let directory_path = std::path::Path::new(matches.value_of("directory").unwrap()).to_owned();
    let script_path = std::path::Path::new(matches.value_of("script").unwrap()).to_owned();

    if !directory_path.exists() || !directory_path.is_dir() {
        println!("Directory does not exist: {}", &directory_path.display());
        return Ok(());
    }

    rocket::build()
        .manage(NobreakState {
            directory: directory_path,
        })
        .mount(
            "/",
            rocket::routes![handle_index, handle_log, handle_get, handle_shutdown],
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
    println!("Finished nobreak.");
    Ok(())
}
