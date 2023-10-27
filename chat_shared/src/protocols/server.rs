use crate::{
    error::{DeserializerError, SerializerError},
    types::{Deserialize, Serialize},
    utils::{prepare_inner_cursor, read_string_from_buffer},
};
use async_trait::async_trait;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(PartialEq, Debug)]
pub enum ServerMessageType {
    BroadcastMessage,
    AuthenticateToken,
    InvalidEvent,
}

impl From<u8> for ServerMessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::BroadcastMessage,
            1 => Self::AuthenticateToken,
            _ => Self::InvalidEvent,
        }
    }
}

#[derive(Debug, PartialEq, Eq, chat_macro::Serialize, chat_macro::Deserialize)]
#[Belonging(ServerMessageType)]
pub struct BroadcastMessage {
    pub username: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, chat_macro::Serialize, chat_macro::Deserialize)]
#[Belonging(ServerMessageType)]
pub struct AuthenticateToken {
    pub token: String,
}
