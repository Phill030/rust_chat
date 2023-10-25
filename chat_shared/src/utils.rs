use crate::error::DeserializerError;
use std::io::Cursor;
use tokio::io::AsyncReadExt;

macro_rules! read_type {
    ($cursor:expr, $ty:ty) => {{
        use std::io::Read;
        let mut buf = [0; std::mem::size_of::<$ty>()];
        $cursor
            .read_exact(&mut buf)
            .await
            .expect("Failed to read data");
        <$ty>::from_le_bytes(buf)
    }};
}

macro_rules! read {
    ($cursor:expr, String, $length:expr) => {{
        use std::io::Read;
        let mut buf = vec![0; $length];
        $cursor
            .read_exact(&mut buf)
            .await
            .expect("Failed to read data");
        String::from_utf8(buf).expect("Failed to convert to String")
    }};

    ($cursor:expr, BigString) => {{
        use std::io::Read;
        let str_len = read_type!($cursor, u32);

        // Read string using length
        let mut buf = vec![0; str_len as usize];
        $cursor
            .read_exact(&mut buf)
            .await
            .expect("Failed to read data");
        String::from_utf8(buf).expect("Failed to convert to String")
    }};
}

pub async fn read_string_from_buffer(
    buffer: &mut Cursor<Vec<u8>>,
) -> Result<Option<String>, DeserializerError> {
    let length = buffer.read_u32().await?;
    let mut temp_buffer = vec![0u8; length as usize];
    buffer.read(&mut temp_buffer).await?;

    match String::from_utf8(temp_buffer) {
        Ok(b) => Ok(Some(b)),
        Err(_) => Ok(None),
    }
}

pub async fn prepare_inner_cursor(cursor: &mut Cursor<&[u8]>) -> std::io::Result<Cursor<Vec<u8>>> {
    let inner_length = cursor.read_u32().await?;
    let mut buffer = vec![0u8; inner_length as usize];
    cursor.read(&mut buffer).await?;

    Ok(Cursor::new(buffer))
}
