use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

use warp::Filter;
use warp::http::StatusCode;
use tokio::fs;

#[derive(Clone)]
pub struct CentralState {
    // Shared state between TCP and HTTP handlers
    pub logs: Arc<Mutex<Vec<String>>>,
    pub servers: Arc<Mutex<Vec<String>>>,
    pub running_containers: Arc<Mutex<Vec<String>>>,
}

pub async fn start_http_server(state: CentralState) {
    // Create routes
    println!("Starting HTTP server on 0.0.0.0:3030");
    // GET /logs - retrieve all logs
    let get_logs = warp::path("logs")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_logs_handler);

    // GET /servers - retrieve all servers
    let get_servers = warp::path("servers")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_servers_handler);
    
    // POST /logs - add a log via HTTP
    let post_logs = warp::path("logs")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(post_logs_handler);
    
    // GET /health - health check
    let health = warp::path("health")
        .map(|| "Server is running");
    
    // Combine routes
    let routes = get_logs
        .or(get_servers)
        .or(post_logs)
        .or(health)
        .with(warp::cors().allow_any_origin());

    //println!("HTTP server listening on 127.0.0.1:3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

// Handler for GET /logs
async fn get_logs_handler(_state: CentralState) -> Result<impl warp::Reply, warp::Rejection> {
    match read_log_files("Logs").await {
        Ok(files) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&files),
                StatusCode::OK,
            ))
        }
        Err(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Failed to read log files"
                })),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

// Handler for POST /logs
async fn post_logs_handler(
    body: serde_json::Value,
    state: CentralState
) -> Result<impl warp::Reply, warp::Rejection> {
    let message = body.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    if message.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": "Log message cannot be empty"
            })),
            StatusCode::BAD_REQUEST,
        ));
    }

    let mut logs = state.logs.lock().await;
    logs.push(format!("HTTP: {}", message));
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "status": "ok",
            "message": "Log added"
        })),
        StatusCode::CREATED,
    ))
}

// Handler for GET /servers
async fn get_servers_handler(_state: CentralState) -> Result<impl warp::Reply, warp::Rejection> {
    let servers_file = "Logs/servers.txt";
    
    let servers = match std::fs::read_to_string(servers_file) {
        Ok(content) => content.lines().map(|line| line.to_string()).collect::<Vec<String>>(),
        Err(_) => Vec::new(),
    };
    
    Ok(warp::reply::with_status(
        warp::reply::json(&servers),
        StatusCode::OK,
    ))
}

// Helper to pass state to handlers
fn with_state(state: CentralState) -> impl Filter<Extract = (CentralState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

// Recursively read all log files from a directory
async fn read_log_files(dir_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut dir_queue = vec![PathBuf::from(dir_path)];

    while let Some(current_dir) = dir_queue.pop() {
        if let Ok(mut entries) = fs::read_dir(&current_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    // Only include .log files, skip other files like servers.txt
                    if path.extension().and_then(|ext| ext.to_str()) == Some("log") {
                        // Convert to string path relative to current directory
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                } else if path.is_dir() {
                    dir_queue.push(path);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}