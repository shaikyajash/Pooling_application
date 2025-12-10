mod controllers;
mod middleware;
mod models;
mod routes;
mod utils;
use routes::api::api_routes;

use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use url::Url;
use webauthn_rs::WebauthnBuilder;

use crate::{
    models::local_store::AppState,
    routes::{auth::auth_routes, health::health_check_routes},
    utils::{connect_to_db::connect_to_db, setup_tables::make_tables_if_not_exists},
};

#[tokio::main]
async fn main() {
    // Loading env First
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("⚠️  Warning: Could not load .env file: {}", e);
        eprintln!("Make sure you have a .env file in the project root");
    }

    //Server initializing Code below

    let rp_id = "localhost";
    let origin = Url::parse("http://localhost:5500").expect("Failed to parse origin");

    let webauth = match WebauthnBuilder::new(&rp_id, &origin) {
        Ok(builder) => match builder.rp_name("PollingApp").build() {
            Ok(webauth) => webauth,
            Err(e) => {
                eprintln!("Error building Webauthn: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("Error creating WebauthnBuilder: {}", e);
            return;
        }
    };

    //Connecting to the database
    println!("Connecting to the database...");
    let db_pool = match connect_to_db().await {
        Ok(pool) => pool,
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            return;
        }
    };

    if let Err(e) = make_tables_if_not_exists(&db_pool).await {
        eprintln!("Error setting up database tables: {}", e);
        return;
    }

    // creating application state with webauth instance
    let app_state = AppState::new(webauth, rp_id.to_string(), db_pool);

    //setting up cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new().nest("/health", health_check_routes())
        .nest("/api", api_routes())
        .nest("/auth", auth_routes())
        .with_state(app_state)
        .layer(cors);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:8080").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address to Tcp Listener: {}", e);
            return;
        }
    };

    println!("Server running on http://localhost:8080");

    match axum::serve(listener, app).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            return;
        }
    };

}
