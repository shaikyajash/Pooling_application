use axum::{Router, middleware, routing::get};

use crate::{
    controllers::{protected_test::protected_test},
    middleware::authentication_middleware::require_authentication, models::local_store::AppState,
};

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/protected/test", get(protected_test))
        .route_layer(middleware::from_fn(require_authentication))
}
