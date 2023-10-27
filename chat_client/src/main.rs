use crate::{client::Client, config::config::ConfigManager};

use std::{
    io::{self},
    net::TcpStream,
};
use utils::construct_hwid;

pub mod client;
mod config;
pub mod types;
pub mod utils;

static KEY: &str = "THERESHOULDB3S0MESECRETKEYINHEREBUTRIGHTNOWTHEREISN'T";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let hwid = construct_hwid();

    let config = ConfigManager::initialize_or_create().await.unwrap();

    match TcpStream::connect_timeout(&config.endpoint, config.timeout) {
        Ok(stream) => {
            log::info!("Connected to server");
            Client::start(&stream, &config, &hwid).await?;
        }
        Err(why) => {
            log::error!("Can't connect to endpoint \n {why}");
        }
    }

    Ok(())
}
