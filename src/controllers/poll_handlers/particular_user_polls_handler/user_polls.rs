use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use sqlx::types::Uuid;

use crate::models::local_store::AppState;

#[derive(Debug, Serialize)]
pub struct UserPollSummary {
    pub id: Uuid,
    pub title: String,
    pub is_closed: bool,
    pub total_votes: i32,
    pub created_at: String,
    pub closed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserPollsResponse {
    pub user_id: Uuid,
    pub polls: Vec<UserPollSummary>,
}

pub async fn user_polls(
    Path(user_id): Path<String>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<UserPollsResponse>), (StatusCode, String)> {
    // Parse user_id
    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid user ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid user ID".to_string()));
        }
    };

    // Fetch polls by user
    let polls = match state.db.get_polls_by_user_id(&user_uuid).await {
        Ok(polls) => polls,
        Err(e) => {
            eprintln!("❌ Error fetching polls: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch polls".to_string(),
            ));
        }
    };

    // Build response
    let mut summaries: Vec<UserPollSummary> = Vec::with_capacity(polls.len());
    for poll in polls {
        summaries.push(UserPollSummary {
            id: poll.id,
            title: poll.title,
            is_closed: poll.is_closed,
            total_votes: poll.total_votes,
            created_at: poll.created_at.to_rfc3339(),
            closed_at: poll.closed_at.map(|d| d.to_rfc3339()),
        });
    }

    Ok((
        StatusCode::OK,
        Json(UserPollsResponse {
            user_id: user_uuid,
            polls: summaries,
        }),
    ))
}
