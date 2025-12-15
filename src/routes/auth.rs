use axum::{Router, routing::post};

use crate::{
    controllers::{
        auth_authentication_handlers::{
            auth_authenticate_finish::authenticate_finish,
            auth_authenticate_start::auth_authenticate_start,
        },
        auth_register_handlers::{
            auth_register_finish::auth_register_finish, auth_register_start::auth_register_function,
        },
    },
    models::local_store::AppState,
};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/register/start", post(auth_register_function))
        .route("/register/finish", post(auth_register_finish))
        .route("/authenticate/start", post(auth_authenticate_start))
        .route("/authenticate/finish", post(authenticate_finish))
}
