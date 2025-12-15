use axum::{Router, routing::get};

use crate::{controllers::health::health_get, models::local_store::AppState};

pub fn health_check_routes() -> Router<AppState> {
    Router::new().route("/", get(health_get))
}
