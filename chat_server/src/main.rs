extern crate chat_macro;

use config::config::ConfigManager;
use server::Server;

pub mod config;
pub mod event_handler;
pub mod server;
pub mod types;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // let db = Surreal::new::<Mem>(()).await.unwrap();
    // db.use_ns("chat").use_db("clients").await.unwrap();
    // let db_client = Arc::new(db);

    let config = ConfigManager::initialize_or_create().await.unwrap();
    Server::create(config.endpoint)?;

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
