use crate::{constants::METADATA_SIZE, message::Message};

pub struct MessageReader {
    pub buffer: Vec<u8>,
}

impl MessageReader {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn read(&mut self, data: &[u8]) -> anyhow::Result<Vec<Message>> {
        self.buffer.extend_from_slice(data);
        let mut data = vec![];
        while self.can_parse() {
            let message = self.parse_first()?;
            data.push(message);
        }

        Ok(data)
    }

    fn can_parse(&self) -> bool {
        if self.buffer.len() < METADATA_SIZE {
            return false;
        }
        let length = u16::from_be_bytes([self.buffer[1], self.buffer[2]]);
        self.buffer.len() >= METADATA_SIZE + length as usize
    }

    fn parse_first(&mut self) -> anyhow::Result<Message> {
        let length = u16::from_be_bytes([self.buffer[1], self.buffer[2]]);
        let message_length = METADATA_SIZE + length as usize;
        let message = self.buffer[..message_length].to_vec();
        self.buffer = self.buffer[message_length..].to_vec();

        Message::decode(&message)
    }
}

impl Default for MessageReader {
    fn default() -> Self {
        Self::new()
    }
}