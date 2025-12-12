use axum::{Extension, Json, http::StatusCode};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProtectedResponse {
    pub message: String,
    pub user_id: String,
}

/// Test endpoint to verify authentication works
pub async fn protected_test(
    Extension(user_id): Extension<String>,  // Extracted from JWT by middleware
) -> Result<Json<ProtectedResponse>, (StatusCode, String)> {
    println!("✅ Protected route accessed by: {}", user_id);
    
    Ok(Json(ProtectedResponse {
        message: "You have successfully accessed a protected route!".to_string(),
        user_id,
    }))
}