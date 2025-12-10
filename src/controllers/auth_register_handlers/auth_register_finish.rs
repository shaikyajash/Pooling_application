use axum::{Json, extract::State, http::StatusCode};


use serde::{Deserialize};
// 1. Import the prelude - this contains everything needed for the server to function.

use crate::{
    controllers::auth_helpers,
    models::{local_store::AppState},
};


#[derive(Deserialize)]
pub struct RegisterFinishRequest {
    pub registration_id: String,
    pub credential: serde_json::Value,
}

pub async fn auth_register_finish(
    State(state): State<AppState>,
    Json(req): Json<RegisterFinishRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Validate registration_id
    let registration_id = req.registration_id.trim();
    
    if registration_id.is_empty() {
        eprintln!("❌ Registration ID cannot be empty");
        return Err((
            StatusCode::BAD_REQUEST,
            "Registration ID is required".to_string(),
        ));
    }

    // Retrieve the registration state and username using the registration_id
    let (registration_state, username) = match state
        .store
        .registration_state
        .get(registration_id)
        .await
    {
        Some(data) => {
            // Remove from cache after retrieval (one-time use)
            state.store.registration_state.invalidate(registration_id).await;
            data
        }
        None => {
            eprintln!("❌ Registration state not found or expired");
            return Err((
                StatusCode::BAD_REQUEST,
                "Registration state not found or expired".to_string(),
            ));
        }
    };

    let reg_credential = match serde_json::from_value(req.credential.clone()) {
        Ok(cred) => cred,
        Err(e) => {
            eprintln!("❌ Invalid credential format: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid credential format: {}", e),
            ));
        }
    };

    let passkey = match state
        .webauth
        .finish_passkey_registration(&reg_credential, &registration_state)
    {
        Ok(passkey) => passkey,
        Err(e) => {
            eprintln!("❌ Error finishing registration: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Error finishing registration: {}", e),
            ));
        }
    };

    // Create user and store passkey atomically - only happens if registration succeeds!
    match auth_helpers::create_user_with_passkey(&username, &passkey, &state).await {
        Ok(_) => {
            println!("✅ Registration successful - user and passkey created");
            Ok(StatusCode::CREATED)
        }
        Err(e) => {
            eprintln!("❌ Error creating user with passkey: {}", e);
            Err((e.status_code(), e.message()))
        }
    }
}
