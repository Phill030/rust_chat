use crate::types::Serializer;

// Server Protocol
#[derive(PartialEq)]
pub enum ServerMessageType {
    BroadcastMessage,
    AuthenticateToken,
}

impl From<u8> for ServerMessageType {
    fn from(value: u8) -> Self {
        match value {
            0 => ServerMessageType::BroadcastMessage,
            1 => ServerMessageType::AuthenticateToken,
            _ => unreachable!(),
        }
    }
}

struct BroadcastMessage {
    sender: String,
    content: String,
}

impl Serializer for BroadcastMessage {
    fn deserialize(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}

//
//

struct AuthenticateToken {
    token: String,
}

impl Serializer for AuthenticateToken {
    fn deserialize(data: &[u8]) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn serialize(&self) -> Vec<u8> {
        todo!()
    }
}
