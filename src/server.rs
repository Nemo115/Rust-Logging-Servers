use std::env::current_dir;
use std::sync::Arc;
use std::path::{Path, PathBuf};
use tokio::fs::{File, self};
use tokio::io::{AsyncWriteExt, AsyncReadExt, BufReader};
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::{log_utils, message_reader::MessageReader, datetime, central_state};


const CHUNK_SIZE: usize = 100_000;

pub struct Server {
    pub host: String,
    pub port: u16,
}

impl Server {
    pub fn new(host: impl Into<String>, port: u16) -> Self{
        Server {
            host: host.into(),
            port,
        }
    }

    // Listens to and receives Message types
    pub async fn run_logging_server(&self, running: Arc<Mutex<bool>>) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await?;
        println!("TCP Server is running on {}:{}", self.host, self.port);
        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("Connection received from {}", addr);

            let running_clone = Arc::clone(&running);
            
            tokio::task::spawn(async move {
                let mut message_reader = MessageReader::new();

                'handler: loop {
                    let mut buffer = [0; 256];
                    let bytes_read = socket.read(&mut buffer).await?;

                    let messages = message_reader.read(&buffer[..bytes_read])?;

                    let mut running_guard = running_clone.lock().await;
                    
                    // Read inputs sent by client
                    for message in messages {
                        if message.content == "exit" {
                            println!("Connection closed by client");
                            break 'handler;
                        }
                        // Command to log the system
                        else if message.content == "syslog" {
                            log_utils::log_system();
                        }
                        // lxc_list
                        else if message.content == "list"{
                            log_utils::lxc_list(); // <-- Does not save any logs atm
                        }
                        // start
                        else if message.content == "start" || message.content == "continue"{
                            *running_guard = true;
                        }
                        // stop / pause
                        else if message.content == "stop" || message.content == "pause"{
                            *running_guard = false;
                        }
                        else {
                            println!("Command not recognised: {:?}", message);
                        }
                    }
                }
                Ok::<(), anyhow::Error>(())
            });
        }
    }

    // Listens to and receives files and metadata
    pub async fn run_storing_server(&self, state: central_state::CentralState) -> anyhow::Result<()>{
        let listener = tokio::net::TcpListener::bind(format!("{}:{}", self.host, self.port)).await?;
        println!("TCP Server is running on {}:{}", self.host, self.port);
        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("Connection received from {}", addr);
            tokio::task::spawn(async move {
                if let Err(e) = Server::handle_receive(socket).await {
                    eprintln!("Connection Error");
                } else{
                    println!("Finished transfer from {}", addr);
                }
                Ok::<(), anyhow::Error>(())
            });
        }
    }
    // Function called when processing a logfile sent from log server to central server
    // This runs on the central server
    pub async fn handle_receive(mut stream: TcpStream) -> anyhow::Result<()> {
        let mut reader = BufReader::new(&mut stream);

        // Read type
        let mut ty = [0u8; 1];
        reader.read_exact(&mut ty).await?;
        if ty[0] != 101u8 {
            anyhow::bail!("unsupported type")
        }

        // Read length
        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf).await?;
        let total_len = u64::from_be_bytes(len_buf);

        // Read filename length and name
        let mut name_len_buf = [0u8; 2];
        reader.read_exact(&mut name_len_buf).await?;
        let name_len = u16::from_be_bytes(name_len_buf) as usize;
        let mut name_buf = vec![0u8; name_len];
        reader.read_exact(&mut name_buf).await?;
        let filename = String::from_utf8(name_buf).unwrap();
        
        // --> Prepare output file path <--
        // Read date time of the file
        println!("filename = {}", filename);
        let mut dt_len_buf = [0u8; 4];
        reader.read_exact(&mut dt_len_buf).await?;
        let dt_len = u32::from_be_bytes(dt_len_buf) as usize;
        let mut dt_buf = vec![0u8; dt_len];
        reader.read_exact(&mut dt_buf).await?;
        let datetime = datetime::DateTime::decode(dt_buf);
        println!("datetime = {}", datetime.to_string());

        // Create Log path and Rotate Logs -- Need some refactoring here
        log_utils::rotate_logs(); // Rotate logs
        let dir_path = log_utils::create_log_dir(datetime); // Log Path

        let mut out_path: PathBuf;
        out_path = PathBuf::from(dir_path);
        
        if !out_path.exists() {
            fs::create_dir_all(&out_path).await?;
        }
        out_path.push(filename.clone());

        // Create file
        let mut out_file = File::create(&out_path).await?;

        // Read exactly total_len bytes and write to file
        let mut remaining = total_len;
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut written: u64 = 0;
        while remaining > 0 {
            let to_read = std::cmp::min(buffer.len() as u64, remaining) as usize;
            reader.read_exact(&mut buffer[..to_read]).await?;
            out_file.write_all(&buffer[..to_read]).await?;
            remaining -= to_read as u64;
            written += to_read as u64;
        }
        out_file.flush().await?;
        //println!("Received and saved {} bytes to {:?}", written, out_path);
        
        // Store server data to JSON file
        log_utils::store_server_name(filename.clone(), out_path.to_string_lossy().to_string());
        
        Ok(())
    }
}