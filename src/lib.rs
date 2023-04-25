use data_encoding::BASE64URL;
use rand::Rng;
use reqwest::{self, Client, ClientBuilder};
use serde_json::{Result, Value};
use sha2::{Digest, Sha256};
use std::{net::IpAddr, str::FromStr};
use tokio::time::{sleep, Duration};

use crate::dirigera_api::{AuthResponse, TokenResponse};

mod dirigera_api;

const CODE_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
const CODE_LENGTH: usize = 128;

/// Token type that represents the auth token for the Dirigera API
type Token = Option<String>;

/// Dirigera struct
/// Can be instantiated by calling Dirigera::new(IpAddr)
#[derive(Debug)]
pub struct Dirigera {
    ip: IpAddr,
    auth_url: String,
    token_url: String,
    base_url: String,
    token: Token,
    code_verifier: String,
    client: Client,
}

impl Dirigera {
    /// Instantiates a new Dirigera struct
    pub fn new(ip: &str) -> Dirigera {
        let base_url = format!("https://{}:8443/v1/", ip.to_string());
        let auth_url = format!("https://{}:8443/v1/oauth/authorize", ip.to_string());
        let token_url = format!("https://{}:8443/v1/oauth/token", ip.to_string());

        Dirigera {
            ip: IpAddr::from_str(&ip).expect("Improper IP entered"),
            auth_url,
            token_url,
            base_url,
            token: Dirigera::check_token_on_init(),
            code_verifier: get_code_verifier(),
            client: ClientBuilder::new()
                .danger_accept_invalid_certs(true) // Won't accept the device cert otherwise
                .build()
                .expect("Unable to build client"),
        }
    }

    /// Acquires an access token for Dirigera struct
    pub async fn get_access_token(&mut self) -> Result<()> {
        // query parameters for access token requests
        let params = [
            ("audience", "homesmart.local"),
            ("response_type", "code"),
            ("code_challenge", &self.create_code_challenge()),
            ("code_challenge_method", "S256"),
        ];

        let auth_response = self
            .client
            .get(&self.auth_url)
            .query(&params)
            .send()
            .await
            .unwrap();

        let body: AuthResponse =
            serde_json::from_str(&auth_response.text().await.unwrap()).unwrap();

        println!("Dirigera auth code is {:?}", body);

        let data = format!(
            "code={}&name={}&grant_type={}&code_verifier={}",
            body,
            "test_app".to_string(),
            "authorization_code".to_string(),
            self.code_verifier.clone(),
        );

        sleep(Duration::new(10, 0)).await;

        let token_response = self
            .client
            .post(&self.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(data)
            .send()
            .await
            .unwrap();

        let token: TokenResponse =
            serde_json::from_str(&token_response.text().await.unwrap()).unwrap();

        println!("Auth token body is {:?}", token);
        self.token = Some(format!("{}", token));

        Ok(())
    }

    /// List all devices on the hub
    pub async fn list_devices(&self) {
        // TODO better error handling
        let token = match &self.token {
            Some(val) => val,
            None => panic!(),
        };

        let dev_res = self
            .client
            .get(format!("{}{}", self.base_url, "devices"))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        let dev_body: Vec<dirigera_api::Device> =
            serde_json::from_str(&dev_res.text().await.unwrap()).unwrap();
        println!("Devices: {:#?}", dev_body);
    }

    /// Will check for an existing access token on init
    fn check_token_on_init() -> Token {
        use dotenv::dotenv;
        dotenv().ok();

        let token = match std::env::var("ACCESS_TOKEN") {
            Ok(val) => {
                println!("Token found: {}", val);
                Some(val)
            }
            Err(_) => {
                println!("No token found");
                None
            }
        };

        token
    }

    /// Creates a Base64URL encoded code_challenge as per RFC7636 for Oauth tokens
    fn create_code_challenge(&self) -> String {
        let mut hasher: Sha256 = Sha256::new();

        hasher.update(&self.code_verifier);

        let hash_digest = hasher.finalize();
        let code_challenge = BASE64URL.encode(&hash_digest).trim_matches('=').to_string();

        code_challenge
    }
}

fn get_code_verifier() -> String {
    let mut rng = rand::thread_rng();

    let code_verifier: String = (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_ALPHABET.len());
            CODE_ALPHABET[idx] as char
        })
        .collect();

    println!("Code Verifier: {:?}", code_verifier);

    code_verifier
}
