use std::sync::{Arc, Mutex};

use warp::Filter;

#[derive(Clone)]
pub struct CentralState {
    // Shared state between TCP and HTTP handlers
    pub logs: Arc<Mutex<Vec<String>>>,
}

pub async fn start_http_server(state: CentralState) {
    // Create routes
    
    // GET /logs - retrieve all logs
    let get_logs = warp::path("logs")
        .and(warp::get())
        .and(with_state(state.clone()))
        .and_then(get_logs_handler);
    
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
        .or(post_logs)
        .or(health)
        .with(warp::cors().allow_any_origin());

    println!("HTTP server listening on 127.0.0.1:3030");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

// Handler for GET /logs
async fn get_logs_handler(state: CentralState) -> Result<impl warp::Reply, warp::Rejection> {
    let logs = state.logs.lock().unwrap();
    Ok(warp::reply::json(&*logs))
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
    
    let mut logs = state.logs.lock().unwrap();
    logs.push(format!("HTTP: {}", message));
    
    Ok(warp::reply::json(&serde_json::json!({
        "status": "ok",
        "message": "Log added"
    })))
}

// Helper to pass state to handlers
fn with_state(state: CentralState) -> impl Filter<Extract = (CentralState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}