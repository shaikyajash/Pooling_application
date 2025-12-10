use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{
    controllers::auth_helpers,
    models::{ local_store::AppState},
};



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

    let authentication_id = authentication_id;


    //Retrieving the authentication State and username using authentication_id
    let (auth_state, user_name) = {

        let mut map = state.store.authentication_states.write().await;
        match map.remove(authentication_id) {
            Some((state, uname)) => (state, uname),
            None => {
                eprintln!("❌ Authentication state not found");
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