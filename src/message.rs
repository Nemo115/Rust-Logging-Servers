use crate::constants::{METADATA_SIZE, PROTOCOL_VERSION};

#[derive(Debug, Clone)]
pub struct Message {
    pub version: u8,
    pub length: u16,
    pub content: String,
}

impl Message {
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let length = content.len() as u16;
        let version = PROTOCOL_VERSION;

        Self {
            version,
            length,
            content,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.push(self.version);
        buffer.extend(&self.length.to_be_bytes());
        buffer.extend(self.content.as_bytes());

        buffer
    }

    pub fn decode(buffer: &[u8]) -> anyhow::Result<Self> {
        if buffer.len() < METADATA_SIZE {
            return Err(anyhow::anyhow!("Invalid message length"));
        }

        let version = buffer[0];
        if version != PROTOCOL_VERSION {
            return Err(anyhow::anyhow!("Invalid protocol version"));
        }

        let length = u16::from_be_bytes([buffer[1], buffer[2]]);
        let content = String::from_utf8(buffer[3..3 + length as usize].to_vec())?;

        Ok(Self {
            version,
            length,
            content,
        })
    }
}