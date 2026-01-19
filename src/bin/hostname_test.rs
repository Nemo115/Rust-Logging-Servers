use std::env;
use lib_setup::{client::Client, message::Message, log_utils, datetime};

/*
    Sends commands to the server logger
*/
#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Connect to server, send file
    println!("hostname = {:?}", log_utils::get_hostname());

    Ok(())
}