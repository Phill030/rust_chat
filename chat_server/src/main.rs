extern crate chat_macro;

use config::config::ConfigManager;
use server::Server;

pub mod config;
pub mod event_handler;
pub mod server;
pub mod types;
pub mod utils;
mod window;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = ConfigManager::initialize_or_create().await.unwrap();
    window::start_window(config);
    Server::create(config.endpoint).unwrap();

    Ok(())
}

#[cfg(test)]
mod tests {
    use chat_shared::{
        protocols::{client::ChatMessage, server::BroadcastMessage},
        types::{Deserialize, Serialize},
    };

    #[tokio::test]
    async fn test_server_serialization() {
        let x = BroadcastMessage {
            username: "USERNAME".to_string(),
            content: "CONTENT".to_string(),
        };
        let serialized = x.serialize().await.unwrap();
        let deserialized = BroadcastMessage::deserialize(&serialized).await.unwrap();
        assert_eq!(deserialized, x, "Deserialization of struct failed!");
    }

    #[tokio::test]
    async fn test_client_serialization() {
        let x = ChatMessage {
            hwid: "HWID".to_string(),
            content: "CONTENT".to_string(),
        };
        let serialized = x.serialize().await.unwrap();
        let deserialized = ChatMessage::deserialize(&serialized).await.unwrap();
        assert_eq!(deserialized, x, "Deserialization of struct failed!");
    }
}

//https://docs.rs/crate/hashcash/latest/source/src/lib.rs
//
//
