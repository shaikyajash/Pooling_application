use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};
use sqlx::types::Uuid;

use crate::controllers::auth_authentication_handlers::auth_authenticate_finish::Claims;

pub async fn require_authentication(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Extract the Authorization header

    let auth_header = req.headers().get(header::AUTHORIZATION).ok_or_else(|| {
        eprintln!("❌ Missing Authorization header");
        StatusCode::UNAUTHORIZED
    })?;

    // 2. Convert header value to &str
    let auth_str = auth_header.to_str().map_err(|_e| {
        eprintln!("❌ Invalid Authorization header encoding");
        StatusCode::UNAUTHORIZED
    })?;

    // 3. Check if it starts with "Bearer "
    let token = auth_str.strip_prefix("Bearer ").ok_or_else(|| {
        eprintln!("❌ Authorization header is not Bearer");
        StatusCode::UNAUTHORIZED
    })?;

    //accessing the secret key from environment variable
    let jwt_secret = match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(e) => {
            eprintln!("❌ Failed to read JWT_SECRET from environment: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

    // 4. Validate the JWT token
    let token_data: TokenData<Claims> = match decode(token, &decoding_key, &Validation::default()) {
        Ok(token_data) => token_data,
        Err(e) => {
            eprintln!("❌ Invalid JWT token: {}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Store username in request extensions for handlers to access
    req.extensions_mut().insert(token_data.claims.sub);

    Ok(next.run(req).await)
}

// Optional middleware for routes that work with or without authentication
pub async fn optional_authentication(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // Try to extract token, but don't fail if it's missing
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
                    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

                    if let Ok(token_data) =
                        decode::<Claims>(token, &decoding_key, &Validation::default())
                    {
                        if let Ok(user_id) = Uuid::parse_str(&token_data.claims.sub) {
                            // Store user_id if valid token exists
                            req.extensions_mut().insert(user_id);
                        }
                    }
                }
            }
        }
    }

    // Continue regardless of authentication status
    Ok(next.run(req).await)
}
