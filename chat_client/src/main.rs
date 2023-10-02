use crate::config::config::ConfigManager;
use chat_shared::error::WriteToStreamError;
use chat_shared::protocols::client::{ChatMessage, RequestAuthentication};
use chat_shared::protocols::server::{AuthenticateToken, BroadcastMessage, ServerMessageType};
use chat_shared::types::{Deserializer, Serializer};
use machineid_rs::{HWIDComponent, IdBuilder};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::{process, thread};
use types::Config;

mod config;
mod types;

static KEY: &str = "1234567890";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let hwid = construct_hwid();

    let config = ConfigManager::initialize_or_create().await.unwrap();
    if let Ok(stream) = TcpStream::connect(config.endpoint) {
        log::info!("Connected to server");
        Client::new(&stream, &config, &hwid).await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Client {
    session_token: String,
}

impl Client {
    async fn new(stream: &TcpStream, config: &Config, hwid: &String) -> io::Result<()> {
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

                    // match deserialized_message {
                    //     Ok(server_message) => match server_message {
                    //         ServerProtocol::BroadcastMessage { sender, content } => {
                    //             log::info!("Received '{content}' from {sender}");
                    //         }
                    //         ServerProtocol::AuthenticateToken { token } => {
                    //             log::info!("Session-Token: {}", token);
                    //         }

                    //         // Every other message
                    //         _ => {
                    //             log::warn!(
                    //                 "Received unknown message from server: {:#?}",
                    //                 server_message
                    //             );
                    //         }
                    //     },
                    //     Err(_) => {
                    //         log::warn!("Error parsing client message");
                    //     }
                    // }

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

        // TODO: Maybe fix this? 😂
        if !write_to_stream(stream, &message)
            .await
            .is_ok_and(|x| x == true)
        {
            log::error!("Error authenticating");
            process::exit(0);
        }
    }
}

fn construct_hwid() -> String {
    let mut builder = IdBuilder::new(machineid_rs::Encryption::SHA256);
    builder
        .add_component(HWIDComponent::CPUID)
        .add_component(HWIDComponent::SystemID);

    match builder.build(KEY) {
        Ok(k) => k,
        Err(why) => {
            log::error!("Unable to construct HWID, {why}");
            process::exit(0);
        }
    }
}

async fn write_to_stream<T>(mut stream: &TcpStream, content: &T) -> Result<bool, WriteToStreamError>
where
    T: Serializer,
{
    let Ok(serialized) = &content.serialize().await else {
        log::error!("Unable to serialize message");
        return Ok(false);
    };

    if stream.write_all(&serialized[..]).is_ok() {
        log::info!("[✔] Message broadcasted!");
        Ok(true)
    } else {
        log::warn!("[❌] There was an error broadcasting the message");
        Ok(false)
    }
}
