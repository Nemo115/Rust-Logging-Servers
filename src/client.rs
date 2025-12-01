use tokio::fs::{File, self};
use tokio::io::{AsyncWriteExt, AsyncReadExt, BufWriter, BufReader};
use tokio::net::TcpStream;

use crate::file_info::FileInfo;
use crate::message::Message;
use crate::log_utils;
use crate::datetime;

const CHUNK_SIZE: usize = 100_000;

pub struct Client {
    pub server_host: String,
    pub server_port: u16,
    pub stream: TcpStream,
}

impl Client {
    pub async fn connect(host: impl Into<String>, port: u16) -> anyhow::Result<Self> {
        let host = host.into();
        let address = format!("{}:{}", host, port);
        let stream = TcpStream::connect(address).await?;

        Ok(Self {
            server_host: host.into(),
            server_port: port,
            stream,
        })
    }

    pub async fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        self.stream.write_all(&message.encode()).await?;
        Ok(())
    }

    pub async fn send_file(&mut self, file_path: String, datetime: datetime::DateTime) -> anyhow::Result<()> {
        let meta = fs::metadata(file_path.clone()).await.expect("File not valid");
        if !meta.is_file() {
            anyhow::bail!("path is not a file: {:?}", file_path);
        }
        
        let file_len = meta.len();
        let filename = file_path
            .split("/")
            .last()
            .unwrap()
            .to_string();

        // Header: 1 byte type + 8 bytes length (big endian) + 2 bytes name length + name bytes
        // Work to package this as a struct
        let mut writer = BufWriter::new(&mut self.stream);
        let file_info: FileInfo = FileInfo::new(file_len, filename, datetime);
        writer.write_all(&file_info.encode()).await?;
        writer.flush().await?;

        // Stream file bytes
        let file = File::open(file_path).await?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut sent: u64 = 0;
        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            writer.write_all(&buffer[..n]).await?;
            sent += n as u64;
        }
        writer.flush().await?;
        println!("Sent {} bytes", sent);

        Ok(())
    }
}