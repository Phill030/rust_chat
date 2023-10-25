extern crate chat_macro;

use chat_shared::{
    protocols::client::ChatMessage,
    types::{Deserialize, Serialize},
};
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

    let x = ChatMessage {
        hwid: "BBBB".to_string(),
        content: "AAAA".to_string(),
    }
    .serialize()
    .await
    .unwrap();

    log::error!("{:?}", x);
    log::error!("{:?}", ChatMessage::deserialize(&x).await.unwrap());

    // let db = Surreal::new::<Mem>(()).await.unwrap();
    // db.use_ns("chat").use_db("clients").await.unwrap();
    // let db_client = Arc::new(db);

    let config = ConfigManager::initialize_or_create().await.unwrap();
    Server::create(config.endpoint)?;

    Ok(())
}
