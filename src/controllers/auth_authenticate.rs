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

    // Store authentication state (automatically expires in 5 minutes)
    state
        .store
        .authentication_states
        .insert(auth_id.clone(), (auth_state, user_name))
        .await;

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




#[derive(Deserialize)]
pub struct AuthenticateFinishRequest {
    pub authentication_id: String,
    pub credential: serde_json::Value,
}

#[derive(Serialize)]
pub struct AuthenticateFinishResponse {
    pub message: String,
    pub user_name: String,
}

pub async fn authenticate_finish(
    State(state): State<AppState>,
    Json(req): Json<AuthenticateFinishRequest>,
) -> Result<Json<AuthenticateFinishResponse>, (StatusCode, String)> {
    // Validate authentication_id
    let authentication_id = req.authentication_id.trim();
    
    if authentication_id.is_empty() {
        eprintln!("❌ Authentication ID cannot be empty");
        return Err((
            StatusCode::BAD_REQUEST,
            "Authentication ID is required".to_string(),
        ));
    }

    //Retrieving the authentication State and username using authentication_id
    let (auth_state, user_name) = match state
        .store
        .authentication_states
        .get(authentication_id)
        .await
    {
        Some(data) => {
            // Remove from cache after retrieval (one-time use)
            state.store.authentication_states.invalidate(authentication_id).await;
            data
        }
        None => {
            eprintln!("❌ Authentication state not found or expired");
            return Err((
                    StatusCode::BAD_REQUEST,
                    "Authentication state not found or expired".to_string(),
                ));
            }
        }
    };


    // parsing the credential from request
    let authentication_credentials = match serde_json::from_value(req.credential.clone()) {
        Ok(cred) => cred,
        Err(e) => {
            eprintln!("❌ Invalid credential format: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid credential format: {}", e),
            ));
        }
    };

    // Verifying the authentication request using webauthn function

    let auth_result = match state
        .webauth
        .finish_passkey_authentication(&authentication_credentials, &auth_state)
    {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Passkey Authentication failed: {}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                format!("Passkey Authentication failed: {}", e),
            ));
        }
    };

    match auth_helpers::update_passkey_counter(&user_name, &auth_result, &state).await {
        Ok(_) => {
            println!("✅ Authentication successful for user: {}", user_name);
            Ok(Json(AuthenticateFinishResponse {
                message: "Login successful".to_string(),
                user_name,
            }))
        }
        Err(e) => {
            eprintln!(" Error updating passkey counter: {}", e);
            Ok(Json(AuthenticateFinishResponse {
                message: "Login successful (counter update failed)".to_string(),
                user_name,
            }))
        }
    }



}