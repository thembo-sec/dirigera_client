use chrono::{Date, DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{map::ValuesMut, Value};
use std::fmt;

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

impl fmt::Display for TokenResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenResponse::AccessToken(a) => write!(f, "{}", a),
            TokenResponse::Error(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Device {
    id: String,
    #[serde(rename = "type")]
    primary_type: DeviceType,
    #[serde(rename = "createdAt")]
    created_at: DateTime<Utc>,
    #[serde(rename = "isReachable")]
    is_reachable: bool,
    #[serde(rename = "lastSeen")]
    last_seen: DateTime<Utc>,
    attributes: Value,
    capabilities: Value,
    room: Option<Value>,
    deviceSet: Vec<Value>,
    #[serde(rename = "remoteLinks")]
    remote_links: Vec<String>,
    #[serde(rename = "isHidden")]
    is_hidden: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
enum DeviceType {
    Outlet,
    Controller,
    Gateway,
    Light,
}
