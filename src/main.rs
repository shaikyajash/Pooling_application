mod controllers;
mod routes;
use routes::health::api_routes;

use axum::Router;

use crate::routes::auth::{auth_routes};

#[tokio::main]
async fn main() {
    
    
    let app = Router::new().nest("/api", api_routes()).nest("/", auth_routes());


    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8080").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address to Tcp Listener: {}", e);
            return;
        }
    };



    println!("Server running on http://127.0.0.1:8080");



    match axum::serve(listener, app).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            return;
        }
    };





    println!("Hello, world!");
}
