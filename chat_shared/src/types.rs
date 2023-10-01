use crate::error::{DeserializerError, SerializerError};
use async_trait::async_trait;

#[async_trait]
pub trait Serializer {
    async fn serialize<'a>(&self) -> Result<Vec<u8>, SerializerError>;
}

#[async_trait]
pub trait Deserializer {
    async fn deserialize<'a>(data: &'a [u8]) -> Result<Option<Self>, DeserializerError>
    where
        Self: Sized;
}
