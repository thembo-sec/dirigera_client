use dotenv::dotenv;
use reqwest::{Client, Error};

mod dirigera_api;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let ip = std::env::var("DIRIGERA_IP").expect("No device IP found");
    let mut d = dirigera_client::Dirigera::new(&ip);

    d.list_devices().await;

    println!("Dirigera: {:?}", d);
    Ok(())
}
