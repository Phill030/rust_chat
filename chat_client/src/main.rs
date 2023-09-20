use crate::config::config::ConfigManager;
use crate::types::ClientProtocol;
use machineid_rs::{HWIDComponent, IdBuilder};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::{process, thread};
use types::{Config, ServerProtocol};

mod config;
mod error;
mod types;

static KEY: &str = "1234567890";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let hwid = construct_hwid();

    let config = ConfigManager::initialize_or_create().await.unwrap();
    if let Ok(stream) = TcpStream::connect(config.endpoint) {
        log::info!("Connected to server");
        Client::new(&stream, &config, &hwid)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct Client {
    session_token: String,
}

impl Client {
    fn new(stream: &TcpStream, config: &Config, hwid: &String) -> io::Result<()> {
        // It is required to send the HWID to the server to authorize with it
        Self::request_authentication(&stream);

        let read_stream = stream.try_clone()?;
        let cloned_config = config.clone();
        thread::spawn(move || {
            Self::read_messages(read_stream, &cloned_config);
        });

        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let trimmed_input = input.trim();
            if trimmed_input.is_empty() {
                continue;
            }

            let message = ClientProtocol::SendMessage {
                hwid: hwid.clone(),
                content: trimmed_input.to_owned(),
            };

            write_to_stream(&stream, &message);
        }
    }

    fn read_messages(mut stream: TcpStream, config: &Config) {
        let mut buffer = vec![0; config.buffer_size];

        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        log::warn!("Server disconnected");
                        break;
                    }

                    let message = String::from_utf8_lossy(&buffer[0..bytes_read]).to_string();

                    let deserialized_message: Result<ServerProtocol, _> =
                        serde_json::from_str(&message);

                    match deserialized_message {
                        Ok(server_message) => match server_message {
                            ServerProtocol::BroadcastMessage { sender, content } => {
                                log::info!("Received '{content}' from {sender}");
                            }
                            ServerProtocol::AuthenticateToken { token } => {
                                log::info!("Session-Token: {}", token);
                            }

                            // Every other message
                            _ => {
                                log::warn!(
                                    "Received unknown message from server: {:#?}",
                                    server_message
                                );
                            }
                        },
                        Err(_) => {
                            log::warn!("Error parsing client message");
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

    fn request_authentication(stream: &TcpStream) {
        let message = ClientProtocol::RequestAuthentication {
            hwid: construct_hwid(),
        };

        if !write_to_stream(stream, &message) {
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

fn write_to_stream<T>(mut stream: &TcpStream, content: &T) -> bool
where
    T: serde::Deserialize<'static> + serde::Serialize, // struct T must have trait Serialize & Deserialize
{
    let serialized_message = serde_json::to_string(&content).expect("Serialization failed");

    if stream.write_all(serialized_message.as_bytes()).is_err() {
        log::warn!("[❌] There was an error broadcasting the message");
        false
    } else {
        log::info!("[✔] Message broadcasted!");
        true
    }
}
