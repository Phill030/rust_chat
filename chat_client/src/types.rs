use std::net::SocketAddr;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ServerProtocol<'a> {
    BroadcastMessage { sender: &'a str, content: &'a str },
    AuthenticateToken { token: &'a str },
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct Config {
    pub endpoint: SocketAddr,
    pub buffer_size: usize,
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint: "127.0.0.1:7878".parse().unwrap(),
            buffer_size: 2048,
            name: format!("User{}", rand::prelude::random::<i16>()),
        }
    }
}
