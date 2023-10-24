use chat_shared::{error::WriteToStreamError, types::Serialize};
use machineid_rs::{HWIDComponent, IdBuilder};
use std::{io::Write, net::TcpStream, process};

use crate::KEY;

pub fn construct_hwid() -> String {
    let mut builder = IdBuilder::new(machineid_rs::Encryption::SHA256);
    builder
        .add_component(HWIDComponent::CPUID)
        .add_component(HWIDComponent::SystemID);

    match builder.build(KEY) {
        Ok(k) => k,
        Err(why) => {
            log::error!("Unable to construct HWID, {why}");
            process::exit(0);
        }
    }
}

pub async fn write_to_stream<T>(
    mut stream: &TcpStream,
    content: &T,
) -> Result<bool, WriteToStreamError>
where
    T: Serialize,
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
