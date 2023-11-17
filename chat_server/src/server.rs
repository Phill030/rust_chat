use crate::{
    event_handler::EventHandler,
    types,
    utils::{check_username, write_to_stream},
};
use chat_shared::{
    protocols::{
        client::{ChangeUsername, ChatMessage, ClientMessageType},
        server::AuthenticateToken,
    },
    types::Deserialize,
};
use std::{
    collections::HashMap,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};
use tokio::sync::Mutex;
use types::Client;

const BUFFER_SIZE: usize = 2048;

#[derive(Debug)]
pub struct Server {
    pub connected_clients: Arc<Mutex<HashMap<String, (TcpStream, Client)>>>,
    pub tcp_listener: TcpListener,
}

impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            connected_clients: self.connected_clients.clone(),
            tcp_listener: self.tcp_listener.try_clone().unwrap(),
        }
    }
}

impl Server {
    pub fn create(endpoint: SocketAddr) -> std::io::Result<()> {
        thread::spawn(move || {
            let connected_clients = Arc::new(Mutex::new(HashMap::new()));
            let tcp_listener = TcpListener::bind(endpoint).unwrap();
            log::info!("Server started @ {:#?}", endpoint);

            let runtime_builder = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            for stream in tcp_listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let connected_clients = connected_clients.clone();
                        log::info!("{} connected,", stream.peer_addr().unwrap());

                        // Each client get's a custom thread

                        runtime_builder.spawn(async move {
                            let mut current_client: Option<(String, String)> = None;

                            {
                                let connected_clients = connected_clients.clone();

                                // We need the HWID here so we can identify the client
                                while current_client.is_none() {
                                    log::info!("Waiting for HWID...");
                                    if let Some(c) = EventHandler::handle_auth(&stream).await {
                                        current_client = Some(c);
                                    } else {
                                        // Remove connection to client?
                                        return;
                                    }
                                }

                                // TODO: Check if HWID already exists, if not create entry with UUID
                                let (client_hwid, client_username) =
                                    current_client.clone().unwrap();
                                log::info!("Found Hwid [{}]", client_hwid);

                                let session_token = uuid::Uuid::new_v4().to_string();
                                let username = check_username(&client_username);
                                let client = Client {
                                    session_token: session_token.clone(),
                                    hwid: client_hwid.clone(),
                                    name: username,
                                };

                                let message = AuthenticateToken {
                                    token: session_token,
                                };
                                write_to_stream(&stream, &message).await.unwrap();

                                connected_clients
                                    .lock()
                                    .await
                                    .insert(client_hwid, (stream.try_clone().unwrap(), client));

                                log::info!(
                                    "Connected clients: {:#?}",
                                    connected_clients.lock().await.len()
                                );
                                Self::handle_connection(&stream, &connected_clients).await;
                            }

                            // This will trigger after the client is disconnected & removes them from the HashMap
                            let mut connected_locked = connected_clients.lock().await;
                            connected_locked.remove(&current_client.clone().unwrap().0);

                            log::info!("Client disconnected");
                        });
                        // We do not join the threads to keep concurrency
                    }
                    Err(why) => {
                        log::error!("Error accepting client connection");
                        log::error!("{}", why);
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_connection(
        mut stream: &TcpStream,
        clients: &Arc<Mutex<HashMap<String, (TcpStream, Client)>>>,
    ) {
        let mut buffer = [0; BUFFER_SIZE];

        loop {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        log::info!("Client disconnected");
                        break;
                    }

                    match ClientMessageType::from(buffer[0]) {
                        ClientMessageType::ChangeUsername => {
                            if let Ok(msg) = ChangeUsername::deserialize(&buffer).await {
                                EventHandler::handle_change_username(msg);
                            }
                        }
                        ClientMessageType::ChatMessage => {
                            if let Ok(msg) = ChatMessage::deserialize(&buffer).await {
                                EventHandler::handle_send_message(msg, clients)
                                    .await
                                    .unwrap();
                            }
                        }

                        _ => EventHandler::handle_unknown_message(),
                    }

                    // Clear the buffer
                    buffer = [0; BUFFER_SIZE];
                }
                Err(why) => {
                    log::error!("{}", why);
                    break;
                }
            }
        }
    }
}
