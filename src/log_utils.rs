use std::collections::HashMap;
use std::{io::Read, process::Command};
use chrono;
use log::LevelFilter;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::read_dir;

use crate::datetime::DateTime;

const LOG_FOLDER: &str = "Logs/";
const ROTATION_MONTHS: u32 = 2;

// Logs whole lxd system, returns log file path
pub fn log_system() -> (String, DateTime){
    // Create and rotate log files
    let (cur_time, fp) = new_log_file(); // Create new log file and pass on the current date time
    del_old_logs(&cur_time, ROTATION_MONTHS); // Rotate and delete past log files
    
    // Get list of containers
    let container_list = lxc_list();
    log::info!("CONTAINER LIST: {:?}", container_list);
    log::info!("OUTPUT: {:?}", storage_pool_status());

    // Log for each container
    for container in container_list {
        let current_container = &container["NAME"];
        log::info!("OUTPUT: {:?}", lxc_ps_aux(current_container));
        log::info!("OUTPUT: {:?}", lxc_info(current_container));
        log::info!("OUTPUT: {:?}", integrity_disk_space(current_container));
    }
    println!("Logged status");
    (fp, cur_time)
}

// Create new log file in directory for the current time
pub fn new_log_file() -> (DateTime, String){
    let dt: DateTime = DateTime::now();

    let dir_path = LOG_FOLDER.to_string() + &dt.year + "/" + &dt.month + "/" + &dt.day;
    let file_path = dir_path.clone() + "/" + &dt.time +".log";

    // Create directory
    match create_dir_all(dir_path) {
        Ok(_) => println!("Successfully created {}", file_path),
        Err(e) => eprint!("Failed to create directory: {}", e)
    }
    simple_logging::log_to_file(file_path.clone(), LevelFilter::Info).unwrap(); 

    (dt, file_path)
}

// Used by central server for creating the directory
pub fn create_log_dir(dt: DateTime) -> String {
    let dir_path = LOG_FOLDER.to_string() + &dt.year + "/" + &dt.month + "/" + &dt.day;
    return dir_path;
}

pub fn rotate_logs() {
    let dt: DateTime = DateTime::now();
    del_old_logs(&dt, ROTATION_MONTHS); // Rotate and delete past log files
}

fn del_dir(dir: &str, time_cutoff: u32) {
    let paths = read_dir(dir).unwrap();
    for path in paths{
        let cur_dir = path.unwrap().path().display().to_string();
        let time: u32 = cur_dir.replace(dir, "").parse().unwrap();
        if time < time_cutoff { // Remove the entire directory
            match remove_dir_all(&cur_dir) {
                Ok(_) => println!("Successfully deleted directory {}", cur_dir),
                Err(e) => eprint!("Failed to delete directory {}: {}", cur_dir, e)
            }
        }
    }
}

// Log rotation function for deleting old logs
pub fn del_old_logs(today: &DateTime, threshold: u32) {
    let cur_year: u32 = today.year.parse().unwrap();
    let cur_month: u32 = today.month.parse().unwrap();
    let mut month_cutoff: u32 = 0;

    if cur_month > threshold {
        month_cutoff = cur_month - threshold;
    }
    
    
    // Scan years first - delete any previous years
    del_dir(LOG_FOLDER, cur_year);

    // Now scan months and delete any months below the cutoff
    if month_cutoff != 0 {
        let months_dir = LOG_FOLDER.to_owned() + &cur_year.to_string() + "/";
        del_dir(&months_dir, month_cutoff);
    }
}

pub fn call_command(args: &[&str]) -> String {
    let mut cmd = Command::new(args[0]);
    for arg in &args[1..] {
        cmd.arg(arg);
    }
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());

    let mut call_output = cmd.spawn().unwrap(); // output handle
    _ = call_output.wait();
    let mut output_buffer = String::new();
    _ = call_output.stdout.unwrap().read_to_string(&mut output_buffer);

    output_buffer
}

// Helper function for lxd commands
// parameters: lxc [list of strings]
fn lxc_command(args: &[&str]) -> String{
    let mut call = vec!["lxc"];
    call.extend_from_slice(args);

    log::info!("CALLED COMMAND: lxc {:?}", args);

    call_command(&call)
}

// read and extract output of ps -- aux 
fn parse_ps_aux(output: &str) -> Vec<HashMap<String, String>> {
    let mut processes = Vec::new();
    let mut lines = output.lines();

    let headers = ["USER", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME"];

    // Skip the header line (if it exists)
    if output.starts_with("USER") {
        lines.next();
    }

    for line in lines {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 11 {
            let mut process = HashMap::new();
            for i in 0..10 {
                process.insert(headers[i].to_string(), fields[i].to_string());
            }
            process.insert("COMMAND".to_string(), fields[10..].join(" ")); // Handle spaces in COMMAND
            processes.push(process);
        }
    }
    processes
}

// For reading terminal outputs that are in the form of tables
// Cannot be used for columns with spaces within values (refer to other helper functions or custom code)
fn parse_tabular_data_table(output: &str) -> Vec<HashMap<String, String>>{
    let mut rows = Vec::new();
    let lines = output.lines();

    // Read and collect the headers
    let mut headers = Vec::new();
    let mut count = 0;

    for line in lines {
        let mut fields: Vec<&str> = line.split_whitespace().collect();
        if count == 0 { // Define the headers as the key values
            headers.append(&mut fields);
        }
        else {
            let mut row = HashMap::new();
            for i in 0..fields.len() {
                row.insert(headers[i].to_string(), fields[i].to_string());
            }
            rows.push(row);
        }
        count+=1;
    }
    rows
}

fn parse_box_data_table(output: &str) -> Vec<HashMap<String, String>>{
    const SKIP_ROWS: usize = 1;

    let mut headers  = Vec::new();
    let mut data_rows: Vec<HashMap<String, String>> = Vec::new(); // Vec of Hashmaps - [{"Name": XXX, "State": XXX, "IPV4": XXX}, {"Name": XXX, "State": XXX, "IPV4": XXX}, ...]
    let mut count = 0;

    for line in output.lines().skip(SKIP_ROWS) {
        if !line.starts_with('|') {
            continue;
        }
        let mut columns: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        
        if count == 0 { // collect the headers
            headers.append(&mut columns);
        }
        else{
            let mut row: HashMap<String, String> = HashMap::new();
        
            for (idx, header) in headers.iter().enumerate(){
                row.insert(header.to_string(), columns[idx].to_string());
            }
            
            data_rows.push(row);
        }
        count += 1;
    }
    data_rows
}

// Container status check
pub fn lxc_list() -> Vec<HashMap<String, String>>{
    let output: String = lxc_command(&["list"]);

    // String Manipulation
    // Read each line and store values into dictionary array
    const SKIP_ROWS: usize = 2;
    const SKIP_ITEM: usize = 1;

    let headers  = ["NAME", "STATE", "IPV4", "IPV6", "TYPE", "SNAPSHOTS"];
    let mut data_rows: Vec<HashMap<String, String>> = Vec::new(); // Vec of Hashmaps - [{"Name": XXX, "State": XXX, "IPV4": XXX}, {"Name": XXX, "State": XXX, "IPV4": XXX}, ...]

    for line in output.lines().skip(SKIP_ROWS) {
        if line.starts_with('|'){
            let columns: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            let mut row: HashMap<String, String> = HashMap::new();
            
            for (idx, header) in headers.iter().enumerate(){
                row.insert(header.to_string(), columns[idx + SKIP_ITEM].to_string());
            }
            
            data_rows.push(row);
        }
    }
    data_rows
}

// Resource Usage
pub fn lxc_info(container_name: &str) -> HashMap<String, String>{
    let output = lxc_command(&["info", container_name]);

    let mut data = HashMap::new();

    // Read first 7 lines of data
    for line in output.lines().take(7) {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();
            data.insert(key, value);
        }
    }
    
    log::info!("DATA: {:?}", data);

    // If container is running extract more data

    // If snapshots exist, read snapshots

    data
}

// Container Process Health
pub fn lxc_ps_aux(container_name: &str) -> Vec<HashMap<String, String>>{
    let output = lxc_command(&["exec", container_name, "--", "ps", "aux"]);
    let processes: Vec<HashMap<String, String>> = parse_ps_aux(&output);
    processes
}

// Network Connectivity - investigate what this does
pub fn network_connectivity(container_name: &str){
    lxc_command(&["exec", container_name, "--", "ping", "-c", "3", "8.8.8.8"]);

}

// File System Integrity and Disk Space
pub fn integrity_disk_space(container_name: &str) -> Vec<HashMap<String, String>>{
    let output = lxc_command(&["exec", container_name, "--", "df", "-h"]);
    //lxc_command(&["exec", container_name, "--", "du", "-sh", "/var/log"]); // Check for specific directory
    let data = parse_tabular_data_table(&output);
    data
}

// Log File Health
pub fn log_file_health(container_name: &str){
    let output = lxc_command(&["exec", container_name, "--", "tail", "-n", "100", "/var/log/syslog"]);
    // Understand the data output
}

// LXD Storage Pool Status
pub fn storage_pool_status() -> Vec<HashMap<String, String>>{
    let output = lxc_command(&["storage", "list"]);
    let data = parse_box_data_table(&output);
    data
}

// Network Interface and DNS
pub fn network_interface_dns(container_name: &str){
    lxc_command(&["exec", container_name, "--", "ip", "a"]);
    lxc_command(&["exec", container_name, "--", "cat", "/etc/resolv.conf"]);
    // ifconfig
}

// Backup Verification
pub fn backup_verification(container_name: &str){
    lxc_command(&["exec", container_name, "backup.tar.gz"]);
}

// Snapshot Management
pub fn snapshot_management(container_name: &str){
    lxc_command(&["snapshot", container_name]);
    lxc_command(&["info", container_name]);
}