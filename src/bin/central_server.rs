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
        logs: Arc::new(Mutex::new(Vec::new())),
        servers: Arc::new(Mutex::new(Vec::new())),
    };
    let tcp_state = state.clone();
    // Establish TCP Server
    let server = Server::new("0.0.0.0", 5000); // original port is 8080, changed to 5000 for multiple hosts

    // Run both servers concurrently
    tokio::select! {
        result = server.run_storing_server(state.clone()) => result?,
        _ = start_http_server(state) => {},
    }

    Ok(())
}