use crate::{
    error::{DeserializerError, SerializerError},
    types::{Deserialize, Serialize},
    utils::{prepare_inner_cursor, read_string_from_buffer},
};
use async_trait::async_trait;
use std::io::Cursor;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(PartialEq, Debug)]
pub enum ClientMessageType {
    ChatMessage,
    ChangeUsername,
    RequestAuthentication,
    InvalidEvent,
}

impl From<u8> for ClientMessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::ChatMessage,
            1 => Self::ChangeUsername,
            2 => Self::RequestAuthentication,
            _ => Self::InvalidEvent,
        }
    }
}

#[derive(Debug, PartialEq, Eq, chat_macro::Serialize, chat_macro::Deserialize)]
#[Belonging(ClientMessageType)]
pub struct ChatMessage {
    pub hwid: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, chat_macro::Serialize, chat_macro::Deserialize)]
#[Belonging(ClientMessageType)]

pub struct ChangeUsername {
    pub hwid: String,
    pub new_username: String,
}

#[derive(Debug, PartialEq, Eq, chat_macro::Serialize, chat_macro::Deserialize)]
#[Belonging(ClientMessageType)]
pub struct RequestAuthentication {
    pub hwid: String,
    pub name: String,
}
