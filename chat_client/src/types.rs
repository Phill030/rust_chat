use std::net::SocketAddr;

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
