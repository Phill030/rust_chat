use std::time::SystemTime;
use tokio::io::AsyncReadExt;

use crate::error::DeserializerError;

pub fn time_in_seconds() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX_EPOCH!"),
    }
}

pub async fn read_string_from_buffer(
    buffer: &mut std::io::Cursor<Vec<u8>>,
) -> Result<Option<String>, DeserializerError> {
    let length = buffer.read_u32().await?;
    let mut temp_buffer = vec![0u8; length as usize];
    buffer.read(&mut temp_buffer).await?;

    match String::from_utf8(temp_buffer) {
        Ok(b) => Ok(Some(b)),
        Err(_) => Ok(None),
    }
}
