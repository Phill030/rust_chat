use std::{error::Error, fmt::Debug};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unable to access file!")]
    IO(#[from] tokio::io::Error),

    #[error("Unable to convert config to string!")]
    TOML(#[from] toml::ser::Error),
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
