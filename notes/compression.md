# Compression

## [Lz4](https://crates.io/crates/lz4_flex)

Lz4 has a fast packing/unpacking speed while providing a safe way to do so in Rust. It's compression ratio is ~ above average which makes it even better.
Otherwise I would've picked Brotli as compression algorithm.

## Old Serialization & Deserialization

```rs
#[rustfmt::skip]
#[async_trait]
impl Serialize for ChangeUsername {
    async fn serialize(&self) -> Result<Vec<u8>, SerializerError> {
        let mut buffer: Vec<u8> = Vec::new();

        // MessageType
        buffer.write_u8(ClientMessageType::ChangeUsername as u8).await?;

        let mut content_buffer = Vec::new();
        content_buffer.write_u32(u32::try_from(self.hwid.as_bytes().len())?).await?;
        content_buffer.extend(self.hwid.as_bytes());
        //\\
        content_buffer.write_u32(u32::try_from(self.new_username.as_bytes().len())?).await?;
        content_buffer.extend(self.new_username.as_bytes());

        buffer.write_u32(crc32fast::hash(&content_buffer[..])).await?;
        buffer.write_u32(u32::try_from(content_buffer.len())?).await?;
        buffer.append(&mut content_buffer);
        return Ok(buffer);
    }
}
```

```rs
#[rustfmt::skip]
#[async_trait]
impl Deserialize for BroadcastMessage {
    async fn deserialize<'a>(data: &'a [u8]) -> Result<Self, DeserializerError>
    where
        Self: Sized,
    {
        if data.is_empty() {
            return Err(DeserializerError::InvalidBufferLength);
        }
        let mut data = Cursor::new(data);

        let msg_type = data.read_u8().await?;
        let message_type = ServerMessageType::from(msg_type);
        if message_type != ServerMessageType::BroadcastMessage {
            return Err(DeserializerError::InvalidMessageType);
        }

        let checksum = data.read_u32().await?;
        // TODO: Compare checksums

        let mut inner_cursor = prepare_inner_cursor(&mut data).await?;
        let username = read_string_from_buffer(&mut inner_cursor).await?;
        let content = read_string_from_buffer(&mut inner_cursor).await?;

        if username.is_none() || content.is_none() {
            return Err(DeserializerError::InvalidData);
        }

        return Ok(Self {
            username: username.unwrap(),
            content: content.unwrap(),
        });
    }
}
```
