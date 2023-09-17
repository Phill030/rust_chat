use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::{process, thread};

use bpaf::{construct, short, OptionParser, Parser};
use machineid_rs::{HWIDComponent, IdBuilder};
use types::ServerProtocol;

use crate::types::ClientProtocol;

mod types;

static KEY: &str = "1234567890";

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Opt {
    name: String,
    addr: SocketAddr,
}

fn opts() -> OptionParser<Opt> {
    let name = short('n')
        .long("name")
        .help("Your username")
        .argument("String")
        .fallback(format!("User{}", rand::prelude::random::<i32>()));

    let addr = short('a')
        .long("addr")
        .help("Change your address to connect to")
        .argument("SocketAddr")
        .fallback("0.0.0.0:7878".parse().unwrap());

    construct!(Opt { name, addr })
        .to_options()
        .footer("Helo")
        .descr("Hmmm")
}

fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let stream = TcpStream::connect("127.0.0.1:7878").expect("Can't connect to server!");
    log::info!("Connected to server");
    // Send authentication message to server
    request_authentication(&stream);

    let read_stream = stream.try_clone().expect("Clone failed");
    thread::spawn(move || {
        read_messages(read_stream);
    });

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            continue;
        }

        let message = ClientProtocol::SendMessage {
            hwid: construct_hwid(), //TODO: replace
            content: trimmed_input.to_owned(),
        };

        write_to_stream(&stream, message);
    }
}

fn read_messages(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

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
                            log::info!("Received message {content} from {sender}");
                        }
                        ServerProtocol::AuthenticateToken { token } => {
                            log::info!("Server-Verified Token: {}", token);
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
                        log::warn!("Error parsing client message")
                    }
                }

                // Clear the buffer
                buffer = [0; 1024];
            }
            Err(why) => {
                println!("Error reading from server! {}", why);
                break;
            }
        }
    }
}

fn request_authentication(mut stream: &TcpStream) -> () {
    let message = ClientProtocol::RequestAuthentication {
        hwid: construct_hwid(),
    };

    if !write_to_stream(&stream, message) {
        log::error!("Error authenticating");
        process::exit(0);
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
            log::error!("Cannot construct HWID, {why}");
            process::exit(0);
        }
    }
}

fn write_to_stream<T>(mut stream: &TcpStream, content: T) -> bool
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
