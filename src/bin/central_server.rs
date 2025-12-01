use tokio::sync::Mutex;
use std::sync::Arc;

use lib_setup::server::Server;
/**
 * Receives files from servers
 * Sends messages to servers
 */
#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Establish TCP Server
    let running = Arc::new(Mutex::new(true));
    let server = Server::new("0.0.0.0", 5000); // original port is 8080, changed to 5000 for multiple hosts
    server.run_storing_server().await?;
    // Take in connections and file transfer sockets
    
    // Receive file and do log rotations

    Ok(())
}