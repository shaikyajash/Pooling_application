use axum::{Extension, Json, extract::{Path, State}, http::StatusCode};
use serde::Serialize;
use sqlx::types::Uuid;

use crate::{controllers::poll_handlers::create_poll_helpers, models::{local_store::AppState, polls::Poll}};


#[derive(Debug, Serialize)]
pub struct ResetPollResponse {
    pub message: String,
    pub poll: Poll,
}

pub async fn reset_poll_handler(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
    Extension(user_id): Extension<String>,
) -> Result<(StatusCode, Json<ResetPollResponse>), (StatusCode, String)> {

  // Parse poll_id from String to Uuid
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid poll ID: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid poll ID".to_string(),
            ));
        }
    };


      // Parse user_id from String to Uuid
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid user ID: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid user ID".to_string(),
            ));
        }
    };

    // Check if poll exists
    let poll = match create_poll_helpers::get_poll_by_id(&poll_uuid, &state).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Poll not found: {}", e);
            return Err((
                StatusCode::NOT_FOUND,
                "Poll not found".to_string(),
            ));
        }
    };

      // Check if user is the poll creator
    if poll.creator_id != user_uuid {
        return Err((
            StatusCode::FORBIDDEN,
            "Only the poll creator can reset votes".to_string(),
        ));
    }

      // Check if poll is closed
    if poll.is_closed {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot reset a closed poll".to_string(),
        ));
    }


    // Reset the poll votes
    if let Err(e) = create_poll_helpers::reset_poll_votes(&poll_uuid, &state).await {
        eprintln!("❌ Error resetting poll votes: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error resetting poll votes".to_string(),
        ));
    }

    // Get updated poll data
    let updated_poll = match create_poll_helpers::get_poll_by_id(&poll_uuid, &state).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Error fetching updated poll: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching poll data".to_string(),
            ));
        }
    };


println!("✅ Poll '{}' reset successfully by creator {}", updated_poll.title, user_uuid);

    Ok((
        StatusCode::OK,
        Json(ResetPollResponse {
            message: "Poll votes reset successfully".to_string(),
            poll: updated_poll,
        }),
    ))

    
}