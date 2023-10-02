use crate::{types::Client, write_to_stream};
use chat_shared::{
    error::WriteToStreamError,
    protocols::{
        client::{ChangeUsername, ChatMessage, ClientMessageType, RequestAuthentication},
        server::BroadcastMessage,
    },
    types::Deserializer,
};
use std::{collections::HashMap, io::Read, net::TcpStream, process, sync::Arc};
use tokio::sync::Mutex;

pub struct EventHandler;

impl EventHandler {
    pub async fn handle_send_message(
        chat_message: ChatMessage,
        clients: &Arc<Mutex<HashMap<String, (TcpStream, Client)>>>,
    ) -> Result<(), WriteToStreamError> {
        let lock = clients.lock().await;

        for (client_hwid, (client_stream, c)) in &*lock {
            if client_hwid.eq(&chat_message.hwid) {
                log::info!("{} --> {}", c.name, chat_message.content.clone());
                continue;
            }

            // The message which get's sent to everyone else
            let message = BroadcastMessage {
                hwid: chat_message.hwid.to_string(),
                content: chat_message.content.to_string(),
            };
            write_to_stream(client_stream, &message).await?;

            return Ok(());
        }

        return Ok(());
    }

    pub async fn handle_auth(mut stream: &TcpStream) -> Option<(String, String)> {
        // Doesn't need to be bigger since it's just the HWID Authorization event
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    log::info!("Client disconnected");
                    process::exit(0);
                }

                match ClientMessageType::from(buffer[0]) {
                    ClientMessageType::RequestAuthentication => {
                        let message = RequestAuthentication::deserialize(&buffer).await;

                        let msg = match message {
                            Ok(m) => m,
                            Err(why) => {
                                panic!("{why}");
                            }
                        };

                        match msg {
                            Some(m) => return Some((m.hwid.to_string(), m.name.to_string())),
                            None => None,
                        }
                    }
                    _ => {
                        log::error!("Received invalid event before authentication");
                        None
                    }
                }
            }

            Err(why) => {
                log::error!("Unable to read from stream! {why}");
                process::exit(0);
            }
        }
    }

    pub fn handle_change_username(change_username: ChangeUsername) {
        log::info!(
            "{} changed their username to {}",
            change_username.hwid,
            change_username.new_username
        );
    }

    pub fn handle_unknown_message() {
        log::warn!("Received unknown message");
    }
}
