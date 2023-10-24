use std::{
    io::{self, Read},
    net::TcpStream,
    process,
};

use chat_shared::{
    protocols::client::{ChatMessage, RequestAuthentication},
    protocols::server::{AuthenticateToken, BroadcastMessage, ServerMessageType},
    types::Deserialize,
};

use crate::{
    types::Config,
    utils::{construct_hwid, write_to_stream},
};

#[derive(Clone, Debug)]
pub struct Client {
    session_token: Option<String>,
}

impl Client {
    pub async fn start(stream: &TcpStream, config: &Config, hwid: &String) -> io::Result<()> {
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
                            let message = AuthenticateToken::deserialize(&buffer).await.unwrap();

                            log::info!("Session-Token: {}", message.token);
                        }
                        ServerMessageType::BroadcastMessage => {
                            // TODO: Something OnError
                            let message = BroadcastMessage::deserialize(&buffer).await.unwrap();

                            log::info!("{} --> {}", message.username, message.content);
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
