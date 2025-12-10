use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, TokenData, Validation, decode};

use crate::{controllers::auth_authentication_handlers::auth_authenticate_finish::Claims};

pub async fn require_authentication(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the Authorization header

    let auth_header = req.headers().get(header::AUTHORIZATION).ok_or_else(|| {
        eprintln!("❌ Missing Authorization header");
        StatusCode::UNAUTHORIZED
    })?;

    // 2. Convert header value to &str
    let auth_str = auth_header.to_str().map_err(|_| {
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
