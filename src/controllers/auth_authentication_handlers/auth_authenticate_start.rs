use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

use crate::{
    controllers::auth_helpers,
    models::{errors::AppError, local_store::AppState},
};

#[derive(Deserialize)]
pub struct AuthenticateStartRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct AuthenticateStartResponse {
    pub authentication_id: String,
    pub public_key_options: serde_json::Value,
}

pub async fn auth_authenticate_start(
    State(state): State<AppState>,
    Json(req): Json<AuthenticateStartRequest>,
) -> Result<Json<AuthenticateStartResponse>, (StatusCode, String)> {
    // Validate username
    let user_name = req.username.trim();
    
    if user_name.is_empty() {
        eprintln!("❌ Username cannot be empty");
        return Err((
            StatusCode::BAD_REQUEST,
            "Username cannot be empty".to_string(),
        ));
    }

    let user_name = user_name.to_string();

    // Get user's passkey from DB
    let passkey = match auth_helpers::get_user_passkeys(&user_name, &state).await {
        Ok(p) => p,
        
        Err(e) => {
            eprintln!("Error retrieving passkey for user {}: {}", user_name, e);
            return match e {
                AppError::UserNotFound => Err((StatusCode::NOT_FOUND, e.message())),
                // Handle all other errors (DatabaseError, SerializationError, etc.)
                _ => Err((e.status_code(), e.message())),
            };
        }
    }; 

    // Start passkey authentication
    let (challenge_response, auth_state) =
        match state.webauth.start_passkey_authentication(&[passkey]) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error starting authentication: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error starting authentication: {}", e),
                ));
            }
        };

    // Generate unique authentication ID
    let auth_id = Uuid::new_v4().to_string();

    // Store authentication state
    state
        .store
        .authentication_states
        .write()
        .await
        .insert(auth_id.clone(), (auth_state, user_name));

    // Serialize the challenge response
    let public_key_options = match serde_json::to_value(&challenge_response) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error serializing challenge response: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error serializing challenge response: {}", e),
            ));
        }
    };

    Ok(Json(AuthenticateStartResponse {
        authentication_id: auth_id,
        public_key_options,
    }))


}



