use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::types::Uuid;

use crate::models::{local_store::AppState, polls::Poll};

#[derive(Debug, Serialize)]
pub struct ClosePollResponse {
    pub message: String,
    pub poll: Poll,
}

pub async fn close_poll_handler(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<(StatusCode, Json<ClosePollResponse>), (StatusCode, String)> {
    // Parse poll_id from String to Uuid
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid poll ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid poll ID".to_string()));
        }
    };

    // Parse user_id from String to Uuid
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid user ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid user ID".to_string()));
        }
    };

    let poll = match state.db.get_poll_by_id(&poll_uuid).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Poll not found: {}", e);
            return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
        }
    };

    // Check if user is the poll creator
    if poll.creator_id != user_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the poll creator can close the poll".to_string(),
        ));
    }

    // Check if poll is already closed
    if poll.is_closed {
        return Err((
            StatusCode::BAD_REQUEST,
            "Poll is already closed".to_string(),
        ));
    }

    // Close the poll
    let updated_poll = match state.db.close_poll(&poll_uuid).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Error closing poll: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error closing poll".to_string(),
            ));
        }
    };

    println!(
        "✅ Poll '{}' closed successfully by creator {}",
        updated_poll.title, user_uuid
    );

    Ok((
        StatusCode::OK,
        Json(ClosePollResponse {
            message: "Poll closed successfully".to_string(),
            poll: updated_poll,
        }),
    ))
}
