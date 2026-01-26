use std::env;
use lib_setup::{client::Client, message::Message, log_utils, datetime};

#[tokio::main]
pub async fn main() -> anyhow::Result<()>{
    // Connect to server, send file
    //println!("hostname = {:?}", log_utils::get_hostname());
    println!("Running containers: {:?}", log_utils::num_running_containers());

    Ok(())
}