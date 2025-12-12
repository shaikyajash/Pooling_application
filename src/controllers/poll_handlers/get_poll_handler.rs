use axum::{Extension, Json, extract::{Path, State}, http::StatusCode};
use serde::Serialize;
use sqlx::types::Uuid;

use crate::{controllers::poll_handlers::create_poll_helpers, models::{local_store::AppState, polls::{Poll, PollOptionWithPercentage}}};




#[derive(Debug, Serialize)]
pub struct GetPollResponse {
    #[serde(flatten)]
    pub poll: Poll,
    pub options: Vec<PollOptionWithPercentage>,
    pub user_voted_option_id: Option<Uuid>,
}

pub async fn get_poll(
    State(state): State<AppState>,
    Path(poll_id): Path<String>,
    user_id: Option<Extension<Uuid>>,  // Optional - from optional_authentication middleware
) -> Result<(StatusCode, Json<GetPollResponse>), (StatusCode, String)> {


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

     // Get poll options
    let options = match create_poll_helpers::get_poll_options(&poll_uuid, &state).await {
        Ok(options) => options,
        Err(e) => {
            eprintln!("❌ Error fetching poll options: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching poll options".to_string(),
            ));
        }
    };


let mut options_with_percentage = Vec::with_capacity(options.len());


    // Calculate total votes for percentage calculation
    for option in &options {

        let percentage = if poll.total_votes > 0 {
            (option.vote_count as f64 / poll.total_votes as f64) * 100.0
        } else {
            0.0
        };

        options_with_percentage.push(PollOptionWithPercentage {
            option: option.clone(),
            percentage,
        });

    };


      // Check if user voted (if authenticated)
    let user_voted_option_id = if let Some(Extension(user_uuid)) = user_id {
        println!("✅ User authenticated: {}", user_uuid);
        match create_poll_helpers::get_user_vote(&poll_uuid, &user_uuid, &state).await {
            Ok(vote) => {
                println!("✅ User vote found: {:?}", vote);
                vote
            },
            Err(e) => {
                println!("⚠️ Error checking vote: {}", e);
                None
            }
        }
    } else {
        println!("⚠️ No user_id in extensions - user not authenticated");
        None
    };



    println!("user_voted_option_id: {:?}", user_voted_option_id);

    println!("✅ Poll '{}' details fetched", poll.title);

    Ok((
        StatusCode::OK,
        Json(GetPollResponse {
            poll,
            options: options_with_percentage,
            user_voted_option_id,
        }),
    ))

}

