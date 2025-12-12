use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::{controllers::poll_handlers::create_poll_helpers, models::{
    local_store::AppState,
    polls::{Poll, PollOption},
}};

#[derive(Debug, Deserialize)]
pub struct CreatePollRequest {
    pub title: String,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePollResponse {
    pub message: String,
    pub poll: Poll,
    pub options: Vec<PollOption>,
}

pub async fn create_poll_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(payload): Json<CreatePollRequest>,
) -> Result<(StatusCode, Json<CreatePollResponse>), (StatusCode, String)> {



    // Parse user_id from String to Uuid
    let creator_id = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("❌ Invalid user ID: {}", e);
            return Err((StatusCode::BAD_REQUEST, "Invalid user ID".to_string()));
        }
    };


    if let Err(e) = validate_poll_request(&payload.title, &payload.options) {
        eprintln!("❌ Poll validation failed: {}", e);
        return Err((StatusCode::BAD_REQUEST, e));
    }

    ///////////////////finished with validating now let's insert the poll into the database
    //// using transactions so lets start transaction

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("❌ Error starting transaction: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ));
        }
    };

    // Insert the poll using helper
    let poll_id = Uuid::new_v4();
    let poll = match create_poll_helpers::insert_poll(&mut tx, poll_id, &payload.title, creator_id).await {
        Ok(poll) => poll,
        Err(e) => {
            eprintln!("❌ Error creating poll: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating poll".to_string(),
            ));
        }
    };

    // Insert poll options using helper
    let options = match create_poll_helpers::insert_poll_options(&mut tx, poll_id, &payload.options).await {
        Ok(options) => options,
        Err(e) => {
            eprintln!("❌ Error creating poll options: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating poll options".to_string(),
            ));
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        eprintln!("❌ Error committing transaction: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error saving poll".to_string(),
        ));
    }

    println!("✅ Poll '{}' created successfully by user {}", poll.title, creator_id);

    Ok((
        StatusCode::CREATED,
        Json(CreatePollResponse {
            message: "Poll created successfully".to_string(),
            poll,
            options,
        }),
    ))
}





/// Validate poll creation request
pub fn validate_poll_request(title: &str, options: &[String]) -> Result<(), String> {
    // Check title
    if title.trim().is_empty() {
        return Err("Poll title cannot be empty".to_string());
    }

    // Check option count
    if options.len() < 2 {
        return Err("At least two options are required".to_string());
    }

    if options.len() > 10 {
        return Err("Poll cannot have more than 10 options".to_string());
    }

    // Check for empty options
    for option in options {
        if option.trim().is_empty() {
            return Err("Poll options cannot be empty".to_string());
        }
    }

    // Check for duplicate options (case-insensitive)
    let unique_options: std::collections::HashSet<_> =
        options.iter().map(|s| s.trim().to_lowercase()).collect();

    if unique_options.len() != options.len() {
        return Err("Duplicate options are not allowed".to_string());
    }

    Ok(())
}
