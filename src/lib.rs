use data_encoding::BASE64URL;
use oauth2::HttpRequest;
use rand::Rng;
use reqwest::{self, header, Client, ClientBuilder, Method};
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
///
/// Can be instantiated by calling Dirigera::new(IpAddr)
#[derive(Debug, Clone)]
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

        let token = Dirigera::check_token_on_init();

        Dirigera {
            ip: IpAddr::from_str(&ip).expect("Improper IP entered"),
            auth_url,
            token_url,
            base_url,
            token: token.clone(),
            code_verifier: get_code_verifier(),
            client: Dirigera::build_client(token),
        }
    }

    fn build_client(token: Token) -> Client {
        let mut headers = header::HeaderMap::new();
        let auth_token = match token {
            Some(i) => i,
            None => panic!(),
        };

        let mut auth_value =
            header::HeaderValue::from_str(&format!("Bearer {}", &auth_token)).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        let new_client = ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .default_headers(headers)
            .build()
            .expect("Unable to build client");

        new_client
    }

    pub async fn build_request(&self, method: Method) {}

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

    /// List all devices on the hub, returns an array of Devices
    pub async fn list_devices(&self) {
        let dev_res = self
            .client
            .get(format!("{}{}", self.base_url, "home"))
            .send()
            .await
            .unwrap();

        let dev_status = &dev_res.status().clone();
        let dev_head = &dev_res.headers().clone();
        let dev_body: Value = serde_json::from_str(
            &dev_res
                .text()
                .await
                .unwrap_or_else(|error| panic!("Getting body: {:?}", error)),
        )
        .unwrap();

        println!(
            "Status: {:#?}\nHeaders: {:#?}\nDevices: {:#?}",
            dev_status, dev_head, dev_body
        );
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

/// Acquire a code verifier as per Oauth protocol
fn get_code_verifier() -> String {
    let mut rng = rand::thread_rng();

    let code_verifier: String = (0..CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CODE_ALPHABET.len());
            CODE_ALPHABET[idx] as char
        })
        .collect();

    code_verifier
}
