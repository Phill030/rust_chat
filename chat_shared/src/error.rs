use std::{error::Error, fmt::Debug, num::TryFromIntError, string::FromUtf8Error};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unable to access file!")]
    IO(#[from] tokio::io::Error),

    #[error("Unable to convert config to string!")]
    TOML(#[from] toml::ser::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum SerializerError {
    #[error("Unable to serialize from stream")]
    IO(#[from] std::io::Error),
    #[error("Unable to convert between types")]
    Type(#[from] TryFromIntError),
}

#[derive(thiserror::Error, Debug)]
pub enum DeserializerError {
    #[error("Unable to deserialize from stream")]
    IO(#[from] std::io::Error),
    #[error("Unable to convert between types")]
    Type(#[from] TryFromIntError),
    #[error("Received invalid MessageType")]
    InvalidMessageType,
    #[error("Received invalid buffer length")]
    InvalidBufferLength,
    #[error("Received invalid data")]
    InvalidData,
    #[error("Unable to convert to UTF-8")]
    FromUtf8Error(#[from] FromUtf8Error),
}

#[derive(thiserror::Error)]
pub enum WriteToStreamError {
    #[error("Unable to write to stream")]
    IO(#[from] std::io::Error),
}

impl Debug for WriteToStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self)?;
        if let Some(source) = self.source() {
            writeln!(f, "Caused by:\n\t{}", source)?;
        }
        Ok(())
    }
}
