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
    /// A custom name of the client
    pub name: String,
    // pub hwid: String,
    /// A Session-Token is a randomly generated String which changes on every reconnect.
    /// It can be used to validate the session of a client.
    pub session_token: Option<String>,
    /// First connection of the client in seconds since 1970
    pub first_connection: u64,
    /// Last connection of the client in seconds since 1970
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
