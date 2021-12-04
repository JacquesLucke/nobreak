#[derive(serde::Serialize)]
enum OperationMode {
    Update,
    // Check,
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
        log: "/log",
        get: "/get",
    };
    serde_json::to_string(&res).unwrap()
}

#[rocket::post("/log/<key>", data = "<msg>")]
fn handle_log(key: &str, msg: &[u8]) {
    println!("Key: {}", &key);
    for v in msg {
        println!("Value: {}", &v);
    }
}

#[rocket::get("/get/<key>")]
fn handle_get(key: &str) -> &'static [u8] {
    &[70, 71, 72, 73]
}

#[rocket::post("/_shutdown")]
fn handle_shutdown(shutdown: rocket::Shutdown) -> &'static str {
    shutdown.notify();
    "Shutdown"
}

#[tokio::main]
async fn execute_script(script_path: &str, server_url: reqwest::Url) -> anyhow::Result<()> {
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

fn on_liftoff(script_path: String, rocket: &rocket::Rocket<rocket::Orbit>) {
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
        .arg(clap::Arg::with_name("file").required(true).index(1))
        .get_matches();
    let script_path = matches.value_of("file").unwrap().to_owned();

    rocket::build()
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
