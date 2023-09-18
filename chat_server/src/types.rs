use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ServerProtocol {
    BroadcastMessage { sender: String, content: String },
    AuthenticateToken { token: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ClientProtocol {
    SendMessage { hwid: String, content: String },
    ChangeUsername { hwid: String, new_username: String },
    RequestAuthentication { hwid: String },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Client {
    pub name: String,
    pub hwid: String,
    pub session_token: Option<String>,
    pub first_connection: u64,
    pub last_connection: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub endpoint: SocketAddr,
    pub buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "0.0.0.0:7878".parse().unwrap(),
            buffer_size: 2048,
        }
    }
}
