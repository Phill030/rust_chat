use super::server::{AuthenticateToken, BroadcastMessage, ServerMessageType};
use crate::{
    error::{DeserializerError, SerializerError},
    types::{Deserializer, Serializer},
    utils::{prepare_inner_cursor, read_string_from_buffer, time_in_seconds},
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
            .write_u32(u32::try_from(self.hwid.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(u32::try_from(self.content.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.content.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer
            .write_u32(u32::try_from(content_buffer.len())?)
            .await?;
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
            .write_u32(u32::try_from(self.hwid.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(u32::try_from(self.new_username.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.new_username.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer
            .write_u32(u32::try_from(content_buffer.len())?)
            .await?;
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
            .write_u32(u32::try_from(self.hwid.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.hwid.as_bytes());
        //Content
        content_buffer
            .write_u32(u32::try_from(self.name.as_bytes().len())?)
            .await?;
        content_buffer.extend(self.name.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer
            .write_u32(u32::try_from(content_buffer.len())?)
            .await?;
        buffer.append(&mut content_buffer);

        return Ok(buffer);
    }
}

//\\//\\//\\//\\//\\//\\//\\//\\
// CLIENT <- SERVER Impls (Deserialize)
//\\//\\//\\//\\//\\//\\//\\//\\

#[async_trait]
impl Deserializer for BroadcastMessage {
    async fn deserialize<'a>(data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Err(DeserializerError::InvalidBufferLength);
        }
        let mut data = Cursor::new(data);

        let msg_type = data.read_u8().await?;
        let message_type = ServerMessageType::from(msg_type);
        if message_type != ServerMessageType::BroadcastMessage {
            return Err(DeserializerError::InvalidMessageType);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes
        let checksum = data.read_u32().await?;
        // TODO: Compare checksums

        let mut inner_cursor = prepare_inner_cursor(&mut data).await?;
        let hwid = read_string_from_buffer(&mut inner_cursor).await?;
        let content = read_string_from_buffer(&mut inner_cursor).await?;

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
    async fn deserialize<'a>(data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Err(DeserializerError::InvalidBufferLength);
        }
        let mut data = Cursor::new(data);

        let msg_type = data.read_u8().await?;
        let message_type = ServerMessageType::from(msg_type);
        if message_type != ServerMessageType::AuthenticateToken {
            return Err(DeserializerError::InvalidMessageType);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes
        let checksum = data.read_u32().await?;
        // TODO: Compare checksums

        let mut inner_cursor = prepare_inner_cursor(&mut data).await?;
        let token = read_string_from_buffer(&mut inner_cursor).await?;

        if token.is_none() {
            return Ok(None);
        }

        return Ok(Some(Self {
            token: token.unwrap(),
        }));
    }
}

// TODO: Make custom implementation for adding header, content, etc.
