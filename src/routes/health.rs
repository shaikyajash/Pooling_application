use axum::{Router, routing::get};

use crate::controllers::health::health_get;

pub fn api_routes() -> Router {
    Router::new().route("/health", get(health_get))
    // .route("/health", health_post)
}
