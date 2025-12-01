//use std::{thread, time};
use tokio::{time};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;

mod log_utils;
use log_utils::log_system;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    // 2 Threads running
    // 1st thread does logging and sleeping - daemon
    println!("Running logging and sleeping daemon...");
    //let logger_handle = tokio::spawn(sleep_logger());


    // 2nd thread listens for commands and acts on them when receiving them
    println!("Running command listener...");
    //let receiver_handle = tokio::spawn(receiver());

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Echo server running on 127.0.0.1:8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling {}: {}", addr, e);
            }
        });
    }
}



// Second thread listens for commands and acts on them when receiving them
async fn receiver() {
    
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buf = [0u8; 1024];

    loop {
        let n = socket.read(&mut buf).await?;

        if n == 0 {
            // Connection closed
            break;
        }

        // Echo the data back
        socket.write_all(&buf[..n]).await?;
    }

    Ok(())
}