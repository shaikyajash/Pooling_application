use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::models::{
    local_store::AppState,
    polls::{Poll, PollOption},
};

#[derive(Debug, Deserialize)]
pub struct VoteRequest {
    pub option_id: String,
}

#[derive(Debug, Serialize)]
pub struct VoteResponse {
    pub message: String,
    pub poll: Poll,
    pub voted_option: PollOption,
}

pub async fn vote_handler(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<VoteRequest>,
) -> Result<(StatusCode, Json<VoteResponse>), (StatusCode, String)> {
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

    // Parse option_id from String to Uuid
    let option_uuid = match Uuid::parse_str(&payload.option_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid option ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid option ID".to_string()));
        }
    };

    // Check if poll exists
    let poll = match state.db.get_poll_by_id(&poll_uuid).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Poll not found: {}", e);
            return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
        }
    };

    // Check if poll is closed
    if poll.is_closed {
        return Err((StatusCode::BAD_REQUEST, "Poll is closed".to_string()));
    }

    match state.db.get_user_vote(&poll_uuid, &user_uuid).await {
        Ok(Some(_)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "You have already voted on this poll".to_string(),
            ));
        }
        Ok(None) => {
            // User hasn't voted, proceed
        }

        Err(e) => {
            eprintln!("❌ Error checking user vote: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error checking vote status".to_string(),
            ));
        }
    }

    // Cast the vote and get the voted option
    let voted_option = match state
        .db
        .cast_vote(&poll_uuid, &user_uuid, &option_uuid)
        .await
    {
        Ok(option) => option,
        Err(e) => {
            eprintln!("❌ Error casting vote: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error casting vote".to_string(),
            ));
        }
    };

    // Get updated poll data
    let updated_poll = match state.db.get_poll_by_id(&poll_uuid).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Error fetching updated poll: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching poll data".to_string(),
            ));
        }
    };

    println!(
        "✅ Vote cast successfully by user {} on poll '{}'",
        user_uuid, updated_poll.title
    );

    Ok((
        StatusCode::CREATED,
        Json(VoteResponse {
            message: "Vote cast successfully".to_string(),
            poll: updated_poll,
            voted_option,
        }),
    ))
}
