#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rocket::build().mount("/", routes![index]).launch().await?;
    Ok(())
}
