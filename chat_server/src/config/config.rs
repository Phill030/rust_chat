use crate::types::Config;
use chat_shared::error::ConfigError;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
};

const CONFIG_NAME: &str = "server_config.toml";
pub struct ConfigManager;

impl ConfigManager {
    pub async fn initialize_or_create() -> Result<Config, ConfigError> {
        let file = File::open(CONFIG_NAME).await;

        if file.is_err() {
            let config = Config::default();

            match File::create(CONFIG_NAME).await {
                Ok(mut f) => {
                    let pretty_config = toml::to_string_pretty(&config)?;
                    if f.write_all(pretty_config.as_bytes()).await.is_err() {
                        log::error!("Unable to create {CONFIG_NAME}!");
                    }
                }
                Err(why) => {
                    log::error!("Unable to create {CONFIG_NAME}! {why}");
                }
            }

            Ok(config)
        } else {
            let mut contents = vec![];
            file.unwrap().read_to_end(&mut contents).await?;

            let config: Result<Config, toml::de::Error> =
                toml::from_str(std::str::from_utf8(&contents).unwrap());

            let config = match config {
                Ok(conf) => conf,
                Err(why) => {
                    log::error!("Unable to convert config to struct (Using default)! {why}");
                    Config::default()
                }
            };

            Ok(config)
        }
    }
}
