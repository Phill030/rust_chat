use crate::{
    event_handler::EventHandler,
    utils::{check_username, write_to_stream},
};
use chat_shared::{
    protocols::{
        client::{ChangeUsername, ChatMessage, ClientMessageType},
        server::AuthenticateToken,
    },
    types::Deserializer,
};
use config::config::ConfigManager;
use std::{
    collections::HashMap,
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
};
use tokio::sync::Mutex;
use types::Client;

pub mod config;
pub mod event_handler;
pub mod types;
pub mod utils;

const BUFFER_SIZE: usize = 2048;

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

pub struct Server {
    pub connected_clients: Arc<Mutex<HashMap<String, (TcpStream, Client)>>>,
    pub tcp_listener: TcpListener,
}

impl Server {
    pub fn create(endpoint: SocketAddr) -> std::io::Result<Server> {
        let connected_clients = Arc::new(Mutex::new(HashMap::new()));
        let tcp_listener = TcpListener::bind(endpoint)?;
        log::info!("Server started @ {:#?}", endpoint);

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let connected_clients = connected_clients.clone();
                    log::info!("{} connected,", stream.peer_addr()?);

                    // Each client get's a custom thread
                    tokio::spawn(async move {
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
                            let client_some = current_client.clone().unwrap();
                            let session_token = uuid::Uuid::new_v4().to_string();
                            let username = check_username(&client_some.1);

                            let client = Client {
                                session_token: session_token.clone(),
                                hwid: client_some.0.clone(),
                                name: username,
                            };
                            log::info!("Found Hwid [{}]", client_some.0);

                            let message = AuthenticateToken {
                                token: session_token,
                            };
                            write_to_stream(&stream, &message).await.unwrap();

                            connected_clients
                                .lock()
                                .await
                                .insert(client_some.0, (stream.try_clone().unwrap(), client));

                            log::info!("{:#?}", connected_clients.lock().await);

                            Self::handle_connection(&stream, &connected_clients).await;
                        }

                        // This will trigger after the client is disconnected & removes them from the HashMap
                        let mut connected_locked = connected_clients.lock().await;
                        connected_locked.remove(&current_client.clone().unwrap().0);

                        log::info!("Client disconnected");
                        log::info!("{:#?}", connected_locked);
                    });
                    // We do not join the threads because then only one connections works at a time!
                }
                Err(why) => {
                    log::error!("Error accepting client connection");
                    log::error!("{}", why);
                }
            }
        }

        Ok(Server {
            connected_clients,
            tcp_listener,
        })
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
                            let msg = ChangeUsername::deserialize(&buffer).await.unwrap();
                            if msg.is_some() {
                                EventHandler::handle_change_username(msg.unwrap());
                            }
                        }
                        ClientMessageType::ChatMessage => {
                            let msg = ChatMessage::deserialize(&buffer).await.unwrap();
                            if msg.is_some() {
                                EventHandler::handle_send_message(msg.unwrap(), clients)
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
