use axum::{Router, routing::get};

use crate::controllers::auth_register::{ auth_register_function};

pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", get(auth_register_function))
        .route("/authenticate", get(auth_register_function))
        
}
