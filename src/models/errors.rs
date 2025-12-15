use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    UserNotFound,
    UserAlreadyHasPasskey,
    DatabaseError(String),
    SerializationError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::UserAlreadyHasPasskey => StatusCode::CONFLICT,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SerializationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn message(&self) -> String {
        match self {
            AppError::UserNotFound => "User not found".to_string(),
            AppError::UserAlreadyHasPasskey => "User already has a passkey registered".to_string(),
            AppError::DatabaseError(msg) => format!("Database error: {}", msg),
            AppError::SerializationError(msg) => format!("Serialization error: {}", msg),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let message = self.message();

        (
            status,
            Json(json!({
                "error": message,
                "status": status.as_u16()
            })),
        )
            .into_response()
    }
}
