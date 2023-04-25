use std::fmt;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AuthResponse {
    Code(String),
    Error(String),
}

// Implement display for AuthResponse as its getting
// written to the request body when querying the device
impl fmt::Display for AuthResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthResponse::Code(c) => write!(f, "{}", c),
            AuthResponse::Error(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum TokenResponse {
    #[serde(rename = "access_token")]
    AccessToken(String),
    #[serde(rename = "lowercase")]
    Error(String),
}
