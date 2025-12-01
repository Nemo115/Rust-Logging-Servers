use std::env;
use lib_setup::{client::Client, message::Message, log_utils::call_command};

/*
    Sends commands to the server logger
*/
#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Collect and process commands
    let args: Vec<String> = env::args().collect();
    // If no commands are called
    if args.len() < 2 {
        eprintln!("Usage: {} <COMMAND>", args[0]);
        std::process::exit(1);
    }
    let command = &args[1];
    println!("Entered Command: {}", command);
    if (command == "start") {
        println!("Starting logging server / daemon");
    }

    // Connect to server, feed command then exit
    // localhost => 127.0.0.1
    // Miracle max => 198.12.64.18
    // My Laptop => 2403:5812:d483::1004 <===> 159.196.67.230
    let mut client = Client::connect("192.168.68.90", 8080).await?;
    client.send_message(Message::new(command)).await?;
    client.send_message(Message::new("exit")).await?;
    Ok(())
}