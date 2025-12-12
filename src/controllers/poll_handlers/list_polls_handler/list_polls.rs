use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::types::Uuid;

use crate::controllers::poll_handlers::poll_helpers;
use crate::models::local_store::AppState;

#[derive(Debug, Serialize)]
pub struct PollSummary {
    pub id: Uuid,
    pub title: String,
    pub is_live: bool,
    pub total_votes: i32,
    pub created_at: String, // ISO8601
    pub closed_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PollsListResponse {
    pub polls: Vec<PollSummary>,
}

pub async fn list_polls(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<PollsListResponse>), (StatusCode, String)> {
    
    let polls = match poll_helpers::get_all_polls(&state).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ Error fetching polls list: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching polls".to_string(),
            ));
        }
    };

    let mut summaries: Vec<PollSummary> = Vec::with_capacity(polls.len());

    for poll in polls {
        summaries.push(PollSummary {
            id: poll.id,
            title: poll.title,
            is_live: !poll.is_closed,
            total_votes: poll.total_votes,
            created_at: poll.created_at.to_rfc3339(),
            closed_at: poll.closed_at.map(|d| d.to_rfc3339()),
        });
    }

    Ok((StatusCode::OK, Json(PollsListResponse { polls: summaries })))
}
