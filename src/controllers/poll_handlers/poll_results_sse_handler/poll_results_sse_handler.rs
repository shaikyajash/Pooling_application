use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use serde::Serialize;
use sqlx::types::Uuid;
use std::convert::Infallible;
use tokio::time::{Duration, interval};

use crate::models::local_store::AppState;

#[derive(Debug, Serialize, Clone)]
pub struct PollOptionResult {
    pub id: Uuid,
    pub option_text: String,
    pub vote_count: i32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct PollResultsUpdate {
    pub poll_id: Uuid,
    pub title: String,
    pub total_votes: i32,
    pub is_closed: bool,
    pub options: Vec<PollOptionResult>,
}

pub async fn poll_results_sse_handler(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    // Parse poll_id from String to Uuid
    let poll_uuid = match Uuid::parse_str(&poll_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid poll ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid poll ID".to_string()));
        }
    };

    // Verify poll exists before starting SSE stream
    match state.db.get_poll_by_id(&poll_uuid).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("❌ Poll not found: {}", e);
            return Err((StatusCode::NOT_FOUND, "Poll not found".to_string()));
        }
    };

    println!("✅ SSE connection established for poll: {}", poll_uuid);

    // Create manual async stream
    let stream = async_stream::stream! {
        let mut interval_timer = interval(Duration::from_secs(5));  // Increased to 5 seconds

        loop {
            interval_timer.tick().await;

            // Single optimized query to get poll + options
            let (poll, options) = match state.db.get_poll_with_options(&poll_uuid).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("❌ Error fetching poll data in SSE: {}", e);
                    continue;
                }
            };

            // Calculate percentages
            let mut options_with_percentage: Vec<PollOptionResult> = Vec::new();

            for option in options {
                let percentage = if poll.total_votes > 0 {
                    (option.vote_count as f64 / poll.total_votes as f64) * 100.0
                } else {
                    0.0
                };

                let option_result = PollOptionResult {
                    id: option.id,
                    option_text: option.option_text,
                    vote_count: option.vote_count,
                    percentage: (percentage * 100.0).round() / 100.0,
                };

                options_with_percentage.push(option_result);
            }

            let update = PollResultsUpdate {
                poll_id: poll.id,
                title: poll.title,
                total_votes: poll.total_votes,
                is_closed: poll.is_closed,
                options: options_with_percentage,
            };

            // Convert to JSON
            let json = match serde_json::to_string(&update) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("❌ Error serializing SSE data: {}", e);
                    continue;
                }
            };

            // Yield the event
            yield Ok(Event::default().data(json));
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
