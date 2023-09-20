use crate::{
    types::{Client, ClientProtocol, ServerProtocol},
    write_to_stream,
};
use std::{
    collections::HashMap,
    io::Read,
    net::TcpStream,
    process,
    sync::{Arc, Mutex},
};

pub struct EventHandler;

impl EventHandler {
    pub fn handle_send_message(
        hwid: &str,
        content: &str,
        clients: &Arc<Mutex<HashMap<String, (TcpStream, Client)>>>,
    ) {
        match clients.lock() {
            Ok(lock) => {
                for (client_hwid, (client_stream, _)) in &*lock {
                    if client_hwid.eq(&hwid) {
                        log::info!("{} said {}", client_hwid, content.clone());

                        continue;
                    }

                    // The message which get's sent to everyone else
                    let message = ServerProtocol::BroadcastMessage {
                        sender: hwid.to_string(),
                        content: content.to_string(),
                    };

                    write_to_stream(client_stream, &message);
                }
            }
            Err(_) => {
                log::error!("There was an error locking the value");
            }
        }
    }

    pub fn handle_auth(mut stream: &TcpStream) -> Option<String> {
        // Doesn't need to be bigger since it's just the HWID Authorization event
        let mut buffer = [0; 1024];

        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    log::info!("Client disconnected");
                    process::exit(0);
                }

                let incoming_message = String::from_utf8_lossy(&buffer[0..bytes_read]).to_string();

                let deserialized_message: Result<ClientProtocol, _> =
                    serde_json::from_str(&incoming_message);

                match deserialized_message {
                    Ok(client_message) => {
                        if let ClientProtocol::RequestAuthentication { hwid } = client_message {
                            Some(hwid)
                        } else {
                            log::error!("Received invalid event before authentication!");
                            None
                        }
                    }
                    Err(_) => None,
                }
            }

            Err(why) => {
                log::error!("Unable to read from stream! {why}");
                process::exit(0);
            }
        }
    }

    pub fn handle_change_username(hwid: &str, new_username: &str) {
        log::info!("{hwid} changed their username to {new_username}");
    }

    pub fn handle_unknown_message(client_message: ClientProtocol) {
        log::warn!("Received unknown message {:#?}", client_message);
    }
}
