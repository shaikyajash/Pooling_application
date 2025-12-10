use axum::{Extension, Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProtectedResponse {
    pub message: String,
    pub username: String,
}

/// Test endpoint to verify authentication works
pub async fn protected_test(
    Extension(username): Extension<String>,  // Extracted from JWT by middleware
) -> Result<Json<ProtectedResponse>, (StatusCode, String)> {
    println!("✅ Protected route accessed by: {}", username);
    
    Ok(Json(ProtectedResponse {
        message: "You have successfully accessed a protected route!".to_string(),
        username,
    }))
}