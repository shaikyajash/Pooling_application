use axum::{Json, extract::State, http::StatusCode};

// ===================Handlers for Auth Register ===================

/*
 * Webauthn RS auth handlers.
 * These files use webauthn to process the data received from each route, and are closely tied to axum
 */

use serde::{Deserialize, Serialize};
// 1. Import the prelude - this contains everything needed for the server to function.
use webauthn_rs::prelude::*;

use crate::{
    controllers::auth_helpers,
    models::{errors::AppError, local_store::AppState},
};

// 2. The first step a client (user) will carry out is requesting a credential to be
// registered. We need to provide a challenge for this. The work flow will be:
//
//          ┌───────────────┐     ┌───────────────┐      ┌───────────────┐
//          │ Authenticator │     │    Browser    │      │     Site      │
//          └───────────────┘     └───────────────┘      └───────────────┘
//                  │                     │                      │
//                  │                     │     1. Start Reg     │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│
//                  │                     │                      │
//                  │                     │     2. Challenge     │
//                  │                     │◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
//                  │                     │                      │
//                  │  3. Select Token    │                      │
//             ─ ─ ─│◀ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─│                      │
//  4. Verify │     │                     │                      │
//                  │  4. Yield PubKey    │                      │
//            └ ─ ─▶│─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶                      │
//                  │                     │                      │
//                  │                     │  5. Send Reg Opts    │
//                  │                     │─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─▶│─ ─ ─
//                  │                     │                      │     │ 5. Verify
//                  │                     │                      │         PubKey
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │─ ─ ─
//                  │                     │                      │     │ 6. Persist
//                  │                     │                      │       Credential
//                  │                     │                      │◀─ ─ ┘
//                  │                     │                      │
//                  │                     │                      │
//
// In this step, we are responding to the start reg(istration) request, and providing
// the challenge to the browser.

#[derive(Deserialize)]
pub struct RegisterStartRequest {
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterStartResponse {
    pub registration_id: String,
    pub public_key_options: serde_json::Value,
}

pub async fn auth_register_function(
    State(state): State<AppState>,
    Json(req): Json<RegisterStartRequest>,
) -> Result<Json<RegisterStartResponse>, (StatusCode, String)> {
    // Validate username
    let user_name = req.username.trim();
    
    if user_name.is_empty() {
        eprintln!("❌ Username cannot be empty");
        return Err((
            StatusCode::BAD_REQUEST,
            "Username cannot be empty".to_string(),
        ));
    }
    
    if user_name.len() < 3 {
        eprintln!("❌ Username too short: {}", user_name);
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be at least 3 characters".to_string(),
        ));
    }
    
    if user_name.len() > 50 {
        eprintln!("❌ Username too long: {}", user_name);
        return Err((
            StatusCode::BAD_REQUEST,
            "Username cannot exceed 50 characters".to_string(),
        ));
    }

    let user_name = user_name.to_string();

    let display_name = match &req.display_name{
        Some(name)=>name.clone(),
        None=>user_name.clone(),
    };

    // Check if user already exists with a passkey
    if let Err(e) = auth_helpers::check_user_exists(&user_name, &state).await {
        eprintln!("Error checking user existence: {}", e);

        // Match on AppError enum
        return match e {
            AppError::UserAlreadyHasPasskey => Err((StatusCode::CONFLICT, e.message())),

            // Handle all other errors (DatabaseError, SerializationError, etc.)
            _ => Err((e.status_code(), e.message())),
        };
    }

    // Generate a temporary user ID for webauthn registration
    let temp_user_id = Uuid::new_v4();

    //we start registration using webauthn
    let (challenge_creation_response, registration_state) =
        match state.webauth.start_passkey_registration(
            temp_user_id,
            &user_name,
            &display_name, // Use actual display_name instead of username
            None,       // No exclude_credentials needed (1 user = 1 passkey)
        ) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error starting registration: {}", e);

                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Error starting registration: {}", e),
                ));
            }
        };

    //Generating  unique registration id
    let reg_id = Uuid::new_v4().to_string();

    // Store registration state with username (not user_id yet - user not created!)
    state
        .store
        .registration_state
        .write()
        .await
        .insert(reg_id.clone(), (registration_state, user_name.clone()));

    //Sending response back to client
    let public_key_options = match serde_json::to_value(&challenge_creation_response) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error serializing challenge response: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error serializing challenge response: {}", e),
            ));
        }
    };

    Ok(Json(RegisterStartResponse {
        registration_id: reg_id,
        public_key_options,
    }))
}



