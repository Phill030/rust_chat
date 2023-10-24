use crate::error::{DeserializerError, SerializerError};
use async_trait::async_trait;

#[async_trait]
pub trait Serialize {
    async fn serialize(&self) -> Result<Vec<u8>, SerializerError>;
}

#[async_trait]
pub trait Deserialize {
    async fn deserialize<'a>(data: &'a [u8]) -> Result<Self, DeserializerError>
    where
        Self: Sized;
}
