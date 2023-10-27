use std::{io::Write, net::TcpStream};

use chat_shared::{
    error::WriteToStreamError,
    types::{Deserialize, Serialize},
};

pub async fn write_to_stream<T>(
    mut stream: &TcpStream,
    content: &T,
) -> Result<bool, WriteToStreamError>
where
    T: Serialize + Deserialize,
{
    let Ok(serialized) = &content.serialize().await else {
        log::error!("Unable to serialize message");
        return Ok(false);
    };

    if stream.write_all(&serialized[..]).is_ok() {
        log::info!("[✔] Message broadcasted!");
        Ok(true)
    } else {
        log::warn!("[❌] There was an error broadcasting the message");
        Ok(false)
    }
}

pub fn check_username(name: &str) -> String {
    if name.is_empty() || name.len() > 32 || !is_alphanumeric_with_symbols(name) {
        return format!("User{}", rand::prelude::random::<i16>());
    }

    name.to_string()
}

pub fn is_alphanumeric_with_symbols(input: &str) -> bool {
    input
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
}
