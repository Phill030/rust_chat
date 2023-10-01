use super::client::{ChangeUsername, ChatMessage, ClientMessageType, RequestAuthentication};
use crate::{
    error::{DeserializerError, SerializerError},
    types::{Deserializer, Serializer},
    utils::time_in_seconds,
};
use async_trait::async_trait;
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

#[derive(Debug)]
pub struct BroadcastMessage {
    pub hwid: String,
    pub content: String,
}

#[derive(Debug)]
pub struct AuthenticateToken {
    pub token: String,
}

//\\//\\//\\//\\//\\//\\//\\//\\
// SERVER <- CLIENT Impls (Deserialize)
//\\//\\//\\//\\//\\//\\//\\//\\

#[async_trait]
impl Deserializer for ChatMessage {
    async fn deserialize<'a>(mut data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Ok(None);
        }

        let msg_type = data.read_u8().await?;
        let message_type = ClientMessageType::from(msg_type);
        if message_type != ClientMessageType::ChatMessage {
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
impl Deserializer for ChangeUsername {
    async fn deserialize<'a>(mut data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Ok(None);
        }

        let msg_type = data.read_u8().await?;
        let message_type = ClientMessageType::from(msg_type);
        if message_type != ClientMessageType::ChangeUsername {
            return Ok(None);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes

        let checksum = data.read_u32().await?;
        // TODO: Compare checksum

        let msg_length = data.read_u32().await?;
        let mut buffer = vec![0u8; msg_length as usize];
        data.read_buf(&mut buffer).await?;

        // Data length (size always u32)
        let hwid_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let hwid = String::from_utf8(buffer[0..hwid_length].to_vec()).ok();

        let new_username_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let new_username =
            String::from_utf8(buffer[hwid_length..new_username_length].to_vec()).ok();

        if hwid.is_none() || new_username.is_none() {
            return Ok(None);
        }

        return Ok(Some(Self {
            hwid: hwid.unwrap(),
            new_username: new_username.unwrap(),
        }));
    }
}

#[async_trait]
impl Deserializer for RequestAuthentication {
    async fn deserialize<'a>(mut data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized,
    {
        if data.len() < 1 {
            return Ok(None);
        }

        println!("{:?}", data);

        let msg_type = data.read_u8().await?;
        let message_type = ClientMessageType::from(msg_type);
        if message_type != ClientMessageType::RequestAuthentication {
            return Err(DeserializerError::InvalidMessageType);
        }

        // Invalid message (Slow response?)
        let timestamp = data.read_u64().await?;
        // TODO: Check if time is in range of 2 minutes

        let checksum = data.read_u32().await?;
        // TODO: Compare checksum

        let msg_length = data.read_u32().await?;
        let mut buffer = vec![0u8; msg_length as usize];
        data.read_buf(&mut buffer).await?;
        println!("SERVER MSG CONTENT {:?}", buffer);

        // Data length (size always u32)
        let hwid_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let hwid = String::from_utf8(buffer[0..hwid_length].to_vec()).ok();
        println!("SERVER {} {}", hwid_length, hwid.clone().unwrap());

        let name_length = usize::try_from(buffer.as_slice().read_u32().await?)?;
        let name = String::from_utf8(buffer[hwid_length..name_length].to_vec()).ok();
        println!("SERVER {} {}", name_length, name.clone().unwrap());

        if hwid.is_none() || name.is_none() {
            return Ok(None);
        }

        return Ok(Some(Self {
            hwid: hwid.unwrap(),
            name: name.unwrap(),
        }));
    }
}

//\\//\\//\\//\\//\\//\\//\\//\\
// CLIENT <- SERVER Impls (Serialize)
//\\//\\//\\//\\//\\//\\//\\//\\

#[async_trait]
impl Serializer for BroadcastMessage {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer
            .write_u8(ServerMessageType::BroadcastMessage as u8)
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
impl Serializer for AuthenticateToken {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer
            .write_u8(ServerMessageType::AuthenticateToken as u8)
            .await?;

        // CurrentTime
        buffer.write_u64(time_in_seconds()).await?;

        let mut content_buffer = Vec::new();
        // HWID
        content_buffer
            .write_u32(self.token.as_bytes().len() as u32)
            .await?;
        content_buffer.extend(self.token.as_bytes());

        // Information
        let checksum = crc32fast::hash(&content_buffer[..]);
        buffer.write_u32(checksum).await?;

        // Append content_buffer length to main buffer after everything is written
        buffer.write_u32(content_buffer.len() as u32).await?;
        buffer.append(&mut content_buffer);

        return Ok(buffer);
    }
}
