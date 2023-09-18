use crate::types::ServerProtocol;
use bpaf::{construct, short, OptionParser, Parser};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    process,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use types::{Client, ClientProtocol};

mod types;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Opt {
    endpoint: SocketAddr,
}

fn opts() -> OptionParser<Opt> {
    let endpoint = short('e')
        .long("endpoint")
        .help("Override the endpoint clients will connect to")
        .argument("SocketAddr")
        .fallback("0.0.0.0:7878".parse().unwrap());

    construct!(Opt { endpoint })
        .to_options()
        .footer("Copyright (c) 2023 Phill030")
        .descr("Hmm")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let opts = opts().run();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // let db = Surreal::new::<Mem>(()).await.unwrap();
    // db.use_ns("chat").use_db("clients").await.unwrap();
    // let db_client = Arc::new(db);

    Server::create(opts.endpoint).await?;

    Ok(())
}

pub struct Server {
    pub connected_clients: Arc<Mutex<HashMap<String, TcpStream>>>,
    pub tcp_listener: TcpListener,
}

impl Server {
    pub async fn create(endpoint: SocketAddr) -> std::io::Result<Server> {
        let connected_clients = Arc::new(Mutex::new(HashMap::new()));
        let tcp_listener = TcpListener::bind(endpoint)?;
        log::info!("Server started @ {endpoint}");

        for stream in tcp_listener.incoming() {
            match stream {
                Ok(stream) => {
                    let connected_clients = connected_clients.clone();
                    log::info!("{} connected,", stream.peer_addr()?);

                    tokio::spawn(async move {
                        let mut hwid: Option<String> = None;

                        {
                            let connected_clients = connected_clients.clone();

                            // We need the HWID here so we can identify the client
                            while hwid.is_none() {
                                log::info!("Waiting for HWID...");
                                match handle_auth(&stream) {
                                    Some(id) => hwid = Some(id),
                                    None => {}
                                }
                            }
                            // TODO: Check if HWID already exists, if not create entry with UUID
                            let hwid = hwid.clone().unwrap();
                            log::info!("Found Hwid [{}]", hwid);

                            let token = uuid::Uuid::new_v4().to_string();
                            // let current_time = SystemTime::now()
                            //     .duration_since(UNIX_EPOCH)
                            //     .unwrap()
                            //     .as_secs();

                            // let client = Client {
                            //     hwid,
                            //     session_token: Some(token.clone()),
                            //     name: "Username".to_string(),
                            //     first_connection: current_time,
                            //     last_connection: current_time,
                            // };

                            let message = ServerProtocol::AuthenticateToken { token };
                            write_to_stream(&stream, message);

                            connected_clients
                                .lock()
                                .expect("Can't lock clients")
                                .insert(hwid, stream.try_clone().unwrap());

                            log::info!("{:#?}", connected_clients.lock().unwrap());

                            Self::handle_connection(stream, connected_clients)
                                .await
                                .unwrap();
                        }

                        // This will trigger after the client is disconnected & removes them from the HashMap
                        log::info!("Client disconnected");
                        connected_clients
                            .as_ref()
                            .lock()
                            .expect("Unable to lock variable")
                            .remove(&hwid.clone().unwrap());

                        log::info!("{:#?}", connected_clients.lock().unwrap());
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
        stream: TcpStream,
        clients: Arc<Mutex<HashMap<String, TcpStream>>>,
    ) -> std::io::Result<()> {
        let mut buffer = [0; 1024];

        let mut stream_cloned = stream.try_clone()?;
        loop {
            match stream_cloned.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        log::info!("Client disconnected");
                        break;
                    }

                    let incoming_message =
                        String::from_utf8_lossy(&buffer[0..bytes_read]).to_string();

                    let deserialized_message: Result<ClientProtocol, _> =
                        serde_json::from_str(&incoming_message);

                    match deserialized_message {
                        Ok(client_message) => match client_message {
                            ClientProtocol::ChangeUsername { hwid, new_username } => {
                                log::info!("{hwid} changed their username to {new_username}");
                            }

                            ClientProtocol::SendMessage { hwid, content } => {
                                log::info!("{} said {}", hwid.clone(), content.clone());
                                match clients.lock() {
                                    Ok(lock) => {
                                        for (client, client_stream) in lock.iter() {
                                            if client.eq(&hwid) {
                                                continue;
                                            }

                                            // The message which get's sent to everyone else
                                            let message = ServerProtocol::BroadcastMessage {
                                                sender: hwid.clone(),
                                                content: content.clone(),
                                            };
                                            write_to_stream(client_stream, message);
                                        }
                                    }
                                    Err(_) => {
                                        log::error!("There was an error locking the value");
                                    }
                                }
                            }

                            // Every other message
                            _ => {
                                log::warn!("Received unknown message {:#?}", client_message);
                            }
                        },
                        Err(why) => {
                            log::error!("Error parsing client message, {}", why);
                        }
                    }

                    // Clear the buffer
                    buffer = [0; 1024];
                }
                Err(why) => {
                    log::error!("{}", why);
                    break;
                }
            }
        }

        Ok(())
    }
}

fn write_to_stream<T>(mut stream: &TcpStream, content: T)
where
    T: serde::Deserialize<'static> + serde::Serialize, // struct T must have trait Serialize & Deserialize
{
    let serialized_message = serde_json::to_string(&content).expect("Serialization failed");

    if stream.write_all(serialized_message.as_bytes()).is_err() {
        log::warn!("[❌] There was an error broadcasting the message");
    } else {
        log::info!("[✔] Message broadcasted!");
    }
}

fn handle_auth(mut stream: &TcpStream) -> Option<String> {
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
                Ok(client_message) => match client_message {
                    ClientProtocol::RequestAuthentication { hwid } => return Some(hwid),

                    _ => {
                        log::error!("Received invalid event!");
                        return None;
                    }
                },
                Err(_) => {
                    return None;
                }
            }
        }

        Err(why) => {
            log::error!("Unable to read from stream! {why}");
            process::exit(0);
        }
    }
}
