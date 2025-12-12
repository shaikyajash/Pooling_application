use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
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
    pub user_id: String,
    pub token: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Claims {
    pub sub: String,      // user_id (UUID as string)
    pub username:String, // username
    pub exp: usize,       // expiration timestamp
    pub iat: usize,       // issued at
}




pub async fn authenticate_finish(
    State(state): State<AppState>,
    Json(req): Json<AuthenticateFinishRequest>,
) -> Result<(StatusCode, Json<AuthenticateFinishResponse>), (StatusCode, String)> {
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

    let (user_id, username) = match auth_helpers::update_passkey_counter(&user_name, &auth_result, &state).await {
        Ok((id, name)) => {
            println!("✅ Authentication successful for user: {} (ID: {})", name, id);
            (id, name)
        }
        Err(e) => {
            eprintln!(" Error updating passkey counter: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error updating authentication data".to_string(),
            ));
        }
    };

    // Generating JWT token with user_id as subject
    let claims = Claims{
        sub: user_id.to_string(),  // User ID as subject (standard practice)
        username: username.clone(),
        exp:(Utc::now() + Duration::hours(24)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };



    //accessing the secret key from environment variable
    let jwt_secret = match std::env::var("JWT_SECRET"){
        Ok(secret) => secret,
        Err(e) => {
            eprintln!("❌ JWT_SECRET not set in environment: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Server configuration error".to_string(),
            ));
        }
    };


    //imp: &secret would share it as -> &String
    // but we want &[u8] so we use .as_ref()

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) { 
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ Error generating JWT token: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error generating authentication token".to_string(),
            ));
        }
    };

Ok((
    StatusCode::CREATED,
    Json(AuthenticateFinishResponse {
        message: "Login successful".to_string(),
        user_name: username,
        user_id: user_id.to_string(),
        token,
    }),
))


}