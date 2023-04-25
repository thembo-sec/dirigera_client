use reqwest::{Client, Error};
use std::net::{IpAddr, Ipv4Addr};

mod dirigera_api;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut d = dirigera_client::Dirigera::new("192.168.0.123");
    d.get_access_token().await;
    println!("Dirigera: {:?}", d);
    Ok(())
}
