use crate::config::config::ConfigManager;
use chat_shared::{
    protocols::client::{ChatMessage, RequestAuthentication},
    protocols::server::{AuthenticateToken, BroadcastMessage, ServerMessageType},
    types::Deserializer,
};
use std::{
    io::{self, Read},
    net::TcpStream,
    process,
};
use types::Config;
use utils::{construct_hwid, write_to_stream};

mod config;
mod types;
pub mod utils;

static KEY: &str = "THERESHOULDB3S0MESECRETKEYINHEREBUTRIGHTNOWTHEREISN'T";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let hwid = construct_hwid();

    let config = ConfigManager::initialize_or_create().await.unwrap();
    if let Ok(stream) = TcpStream::connect(config.endpoint) {
        log::info!("Connected to server");
        Client::start(&stream, &config, &hwid).await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Client {
    session_token: String,
}

impl Client {
    async fn start(stream: &TcpStream, config: &Config, hwid: &String) -> io::Result<()> {
        // It is required to send the HWID to the server to authorize with it
        Self::request_authentication(&stream, &config).await;

        let read_stream = stream.try_clone()?;
        let cloned_config = config.clone();
        tokio::spawn(async move {
            Self::read_messages(read_stream, &cloned_config).await;
        });

        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let trimmed_input = input.trim();
            if trimmed_input.is_empty() {
                continue;
            }

            let message = ChatMessage {
                hwid: hwid.to_string(),
                content: trimmed_input.to_string(),
            };

            write_to_stream(&stream, &message).await.unwrap();
        }
    }

    async fn read_messages(mut stream: TcpStream, config: &Config) {
        let mut buffer = vec![0; config.buffer_size];

        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        log::warn!("Server disconnected");
                        process::exit(0);
                    }

                    match ServerMessageType::from(buffer[0]) {
                        ServerMessageType::AuthenticateToken => {
                            let message = AuthenticateToken::deserialize(&buffer)
                                .await
                                .unwrap()
                                .unwrap();

                            log::info!("Session-Token: {}", message.token);
                        }
                        ServerMessageType::BroadcastMessage => {
                            // TODO: Something OnError
                            let message = BroadcastMessage::deserialize(&buffer)
                                .await
                                .unwrap()
                                .unwrap();

                            log::info!("Received {} from {}", message.content, message.hwid);
                        }
                        ServerMessageType::InvalidEvent => {
                            log::warn!("Received unknown message from server");
                        }
                    }

                    // Clear the buffer
                    buffer = vec![0; config.buffer_size];
                }
                Err(why) => {
                    println!("Error reading from server! {why}");
                    process::exit(0);
                }
            }
        }
    }

    async fn request_authentication(stream: &TcpStream, config: &Config) {
        let message = RequestAuthentication {
            hwid: construct_hwid(),
            name: config.name.to_string(),
        };

        if !write_to_stream(stream, &message)
            .await
            .is_ok_and(|x| x == true)
        {
            log::error!("Error authenticating");
            process::exit(0);
        }
    }
}
