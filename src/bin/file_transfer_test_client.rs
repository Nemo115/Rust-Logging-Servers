use std::env;
use lib_setup::{client::Client, message::Message, log_utils, datetime};

/*
    Sends commands to the server logger
*/
#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Connect to server, send file
    let mut client = Client::connect("192.168.68.90", 5000).await?;
    client.send_file("/home/leozl/Desktop/rust/privacy_lock/misc/testing_transfer_file_2.txt".to_string(), datetime::DateTime::now()).await?;

    Ok(())
}