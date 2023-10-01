use crate::{
    error::{DeserializerError, SerializerError},
    types::{Deserializer, Serializer},
    utils::time_in_seconds,
};
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::server::{AuthenticateToken, BroadcastMessage, ServerMessageType};

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

#[derive(Debug)]
pub struct ChatMessage {
    pub hwid: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ChangeUsername {
    pub hwid: String,
    pub new_username: String,
}

#[derive(Debug)]
pub struct RequestAuthentication {
    pub hwid: String,
    pub name: String,
}

//\\//\\//\\//\\//\\//\\//\\//\\
// CLIENT -> SERVER Impls (Serialize)
//\\//\\//\\//\\//\\//\\//\\//\\

#[async_trait]
impl Serializer for ChatMessage {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer
            .write_u8(ClientMessageType::ChatMessage as u8)
            .await?;

        // CurrentTime
        buffer.write_u64(time_in_seconds()).await?;

        let mut content_buffer = Vec::new();
        // HWID
        content_buffer
            .write_u32(self.hwid.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(self.content.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.content.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer.write_u32(content_buffer.len() as u32).await?;
        buffer.append(&mut content_buffer);

        return Ok(buffer);
    }
}

#[async_trait]
impl Serializer for ChangeUsername {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer
            .write_u8(ClientMessageType::ChangeUsername as u8)
            .await?;

        // CurrentTime
        buffer.write_u64(time_in_seconds()).await?;

        let mut content_buffer = Vec::new();
        // HWID
        content_buffer
            .write_u32(self.hwid.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(self.new_username.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.new_username.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer.write_u32(content_buffer.len() as u32).await?;
        buffer.append(&mut content_buffer);

        return Ok(buffer);
    }
}

#[async_trait]
impl Serializer for RequestAuthentication {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer
            .write_u8(ClientMessageType::RequestAuthentication as u8)
            .await?;

        // CurrentTime
        buffer.write_u64(time_in_seconds()).await?;

        let mut content_buffer = Vec::new();
        // HWID
        content_buffer
            .write_u32(self.hwid.as_bytes().len() as u32)
            .await?;
        println!("{}, {}", self.hwid, self.hwid.as_bytes().len() as u32);
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(self.name.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.name.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer.write_u32(content_buffer.len() as u32).await?;
        println!("CLIENT CONTENT_BUFFER LEN {}", content_buffer.len() as u32);
        buffer.append(&mut content_buffer);

        return Ok(buffer);
    }
}

//\\//\\//\\//\\//\\//\\//\\//\\
// CLIENT <- SERVER Impls (Deserialize)
//\\//\\//\\//\\//\\//\\//\\//\\

#[async_trait]
impl Deserializer for BroadcastMessage {
    async fn deserialize<'a>(mut data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Ok(None);
        }

        let msg_type = data.read_u8().await?;
        let message_type = ServerMessageType::from(msg_type);
        if message_type != ServerMessageType::BroadcastMessage {
            return Ok(None);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes

        let checksum = data.read_u32().await?;
        // TODO: Compare checksums

        let msg_length = data.read_u32().await?;
        let mut buffer = vec![0u8; msg_length as usize];
        data.read_buf(&mut buffer).await?;

        // Data length (size always u32)
        let hwid_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let hwid = String::from_utf8(buffer[0..hwid_length].to_vec()).ok();

        let content_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let content = String::from_utf8(buffer[hwid_length..content_length].to_vec()).ok();

        if hwid.is_none() || content.is_none() {
            return Ok(None);
        }

        return Ok(Some(Self {
            hwid: hwid.unwrap(),
            content: content.unwrap(),
        }));
    }
}

#[async_trait]
impl Deserializer for AuthenticateToken {
    async fn deserialize<'a>(mut data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Ok(None);
        }

        let msg_type = data.read_u8().await?;
        let message_type = ServerMessageType::from(msg_type);
        if message_type != ServerMessageType::AuthenticateToken {
            return Ok(None);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes

        let checksum = data.read_u32().await?;
        // TODO: Compare checksums

        let msg_length = data.read_u32().await?;
        let mut buffer = vec![0u8; msg_length as usize];
        data.read_buf(&mut buffer).await?;

        // Data length (size always u32)
        let token_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let token = String::from_utf8(buffer[0..token_length].to_vec()).ok();

        if token.is_none() {
            return Ok(None);
        }

        return Ok(Some(Self {
            token: token.unwrap(),
        }));
    }
}
