use tokio::time;
use tokio::sync::Mutex;
use std::sync::Arc;

use lib_setup::log_utils;
use lib_setup::server::Server;
use lib_setup::client::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    // Mutex for starting and stopping logging
    let running = Arc::new(Mutex::new(true)); // checked by daemon
    let running_worker = Arc::clone(&running); // toggled by server

    // 1st thread does logging and sleeping - daemon
    println!("Running logging and sleeping daemon...");
    let logger_handle = tokio::spawn(sleep_logger(running));

    // 2nd thread listens for commands and acts on them when receiving them
    println!("Running command listener server...");
    let server = Server::new("0.0.0.0", 8080);
    server.run_logging_server(running_worker).await?;

    Ok(())
}

// First thread does logging and sleeping
async fn sleep_logger(running: Arc<Mutex<bool>>){
    let four_hours = time::Duration::from_secs(60*60*4);
    let twenty_sec = time::Duration::from_secs(20);

    loop {
        // Check if should run
        let should_run = {
            // Mutex controlled by server
            let running_guard = running.lock().await;
            *running_guard
        };

        if should_run{
            // Log system and send logfile to central server
            let (fp, dt) = log_utils::log_system();
            let mut client = Client::connect("127.0.0.1", 5000).await.unwrap();
            client.send_file(fp, dt).await.unwrap();
            time::sleep(twenty_sec).await;
        } else {
            println!("Paused");
        }
    }
}