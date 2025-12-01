/**
 * Structure and implementation for file transmissions across servers
 */
use crate::log_utils;
use crate::datetime;

pub struct FileInfo {
    pub _type: u8,
    pub f_len: u64,
    pub fn_len: u16,
    pub filename: String,
    pub dt_len: u32,
    pub datetime: datetime::DateTime
}

impl FileInfo {
    pub fn new(file_len: u64, filename: String, datetime: datetime::DateTime) -> Self {
        Self {
            _type: 101,
            f_len: file_len,
            fn_len: filename.len() as u16,
            filename: filename,
            dt_len: datetime.encode().len() as u32,// Figure out how to read datetime from receive
            datetime: datetime
        }
    }
    pub fn encode(self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(self._type.to_be_bytes());
        buffer.extend(self.f_len.to_be_bytes());
        buffer.extend(self.fn_len.to_be_bytes());
        buffer.extend(self.filename.into_bytes());
        buffer.extend(self.dt_len.to_be_bytes());
        buffer.extend(self.datetime.encode());

        buffer
    }
}