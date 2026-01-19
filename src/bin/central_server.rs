use tokio::sync::Mutex;
use std::sync::Arc;

use lib_setup::{central_state::{CentralState, start_http_server}, server::Server};
/**
 * Receives files from servers
 * Sends messages to servers
 */
#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Shared state between TCP and HTTP
    let state = CentralState{
        logs: Arc::new(std::sync::Mutex::new(Vec::new())),
    };
    let tcp_state = state.clone();
    // Establish TCP Server
    let running = Arc::new(Mutex::new(true));
    let server = Server::new("0.0.0.0", 5000); // original port is 8080, changed to 5000 for multiple hosts
    server.run_storing_server(state).await?;
    // Take in connections and file transfer sockets
    
    // Receive file and do log rotations

    // Establish HTTP Server
    start_http_server(state);

    println!("TCP Server listening on 127.0.0.1:5000");
    println!("HTTP Server listening on 127.0.0.1:3030");

    Ok(())
}