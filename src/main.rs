mod config;
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
    routes::{auth::auth_routes, health::health_check_routes, polls::polls_routes},
    utils::{db_helpers::Database, setup_tables::make_tables_if_not_exists},
};

#[tokio::main]
async fn main() {
    // Loading env First
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("⚠️  Warning: Could not load .env file: {}", e);
        eprintln!("Make sure you have a .env file in the project root");
    }

    //Server initializing Code below

    // Read RP_ID and ORIGIN from environment (fallback to sensible defaults)
    let rp_id = match std::env::var("RP_ID") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("⚠️ RP_ID not set in environment - defaulting to 'localhost'");
            "localhost".to_string()
        }
    };

    let origin_str = match std::env::var("ORIGIN") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("⚠️ ORIGIN not set in environment - defaulting to 'http://localhost:3000'");
            "http://localhost:3000".to_string()
        }
    };

    let origin = match Url::parse(&origin_str) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("❌ Failed to parse ORIGIN '{}': {}", origin_str, e);
            return;
        }
    };

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
    let db = match Database::new().await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Database connection error: {}", e);
            return;
        }
    };

    if let Err(e) = make_tables_if_not_exists(&db.pool).await {
        eprintln!("Error setting up database tables: {}", e);
        return;
    }

    // creating application state with webauth instance
    let app_state = AppState::new(webauth, rp_id.to_string(), db);

    //setting up cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/health", health_check_routes())
        .nest("/api", api_routes())
        .nest("/auth", auth_routes())
        .nest("/polls", polls_routes())
        .with_state(app_state)
        .layer(cors);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to address to Tcp Listener: {}", e);
            return;
        }
    };

    println!("Server running on http://localhost:3000");

    match axum::serve(listener, app).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to start server: {}", e);
            return;
        }
    };
}
