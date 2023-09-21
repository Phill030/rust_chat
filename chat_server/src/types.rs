use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ServerProtocol<'a> {
    BroadcastMessage { sender: &'a str, content: &'a str },
    AuthenticateToken { token: &'a str },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum ClientProtocol<'a> {
    SendMessage {
        hwid: &'a str,
        content: &'a str,
    },
    ChangeUsername {
        hwid: &'a str,
        new_username: &'a str,
    },
    RequestAuthentication {
        hwid: &'a str,
        name: &'a str,
    },
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone, Eq, Hash)]
pub struct Client {
    /// A custom name of the client
    pub name: String,
    pub hwid: String,
    /// A Session-Token is a randomly generated String which changes on every reconnect.
    /// It can be used to validate the session of a client.
    pub session_token: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Config {
    pub endpoint: SocketAddr,
    pub buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:7878".parse().unwrap(),
            buffer_size: 2048,
        }
    }
}
