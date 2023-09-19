#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unable to access file!")]
    IO(#[from] tokio::io::Error),

    #[error("Unable to convert config to string!")]
    TOML(#[from] toml::ser::Error),
}
