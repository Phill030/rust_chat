use tokio::io::AsyncWriteExt;

use crate::types::Serializer;

// Client Protocol
#[derive(PartialEq)]
pub enum ClientMessageType {
    ChatMessage,
    UsernameChange,
    AuthenticationRequest,
}
impl From<u8> for ClientMessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => ClientMessageType::ChatMessage,
            1 => ClientMessageType::UsernameChange,
            2 => ClientMessageType::AuthenticationRequest,
            _ => unreachable!(),
        }
    }
}

// ChatMessage
struct ChatMessage {
    hwid: String,
    content: String,
}

impl ChatMessage {
    pub fn new(hwid: String, content: String) -> Self {
        Self { hwid, content }
    }
}

impl Serializer for ChatMessage {
    fn deserialize(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return None;
        }
        let message_type = ClientMessageType::from(data[0]);
        if message_type != ClientMessageType::ChatMessage {
            return None;
        }

        let hwid_bytes = &data[1..30];
        let content_bytes = &data[30..];
        let hwid = String::from_utf8(hwid_bytes.to_vec()).ok()?;
        let content = String::from_utf8(content_bytes.to_vec()).ok()?;

        Some(ChatMessage { hwid, content })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.write_u8(ClientMessageType::ChatMessage as u8);
        buffer.extend(self.hwid.as_bytes());
        buffer.extend(self.content.as_bytes());
        buffer
    }
}

// UsernameChange
struct UsernameChange {
    hwid: String,
    new_username: String,
}
impl UsernameChange {
    pub fn new(hwid: String, new_username: String) -> Self {
        Self { hwid, new_username }
    }
}

impl Serializer for UsernameChange {
    fn deserialize(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

// AuthenticationRequest
struct AuthenticationRequest {
    hwid: String,
    name: String,
}
impl AuthenticationRequest {
    pub fn new(hwid: String, name: String) -> Self {
        Self { hwid, name }
    }
}

impl Serializer for AuthenticationRequest {
    fn deserialize(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}
