use base64::Engine;
use sqlx::{Row, types::Uuid};
use webauthn_rs::prelude::{AuthenticationResult, CredentialID, Passkey};

use crate::models::errors::AppError;
use crate::models::local_store::AppState;

// Check if user already exists with a passkey
pub async fn check_user_exists(user_name: &str, state: &AppState) -> Result<(), AppError> {
    let user = sqlx::query(
        r#"
        SELECT credential_id
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(user_name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| {
        eprintln!("Database error checking user: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    match user {
        Some(row) => {
            // User exists - check if they have a passkey
            let credential_id: Option<String> = row.try_get("credential_id").ok();

            let cred = credential_id.and_then(|cred_id_b64| {
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(&cred_id_b64)
                    .ok()
                    .map(CredentialID::from)
            });

            if cred.is_some() {
                println!("User '{}' already has a passkey", user_name);
                return Err(AppError::UserAlreadyHasPasskey);
            }

            // User exists but no passkey - this shouldn't happen in our new flow
            // but we'll allow re-registration
            println!(
                "User '{}' exists without passkey - allowing registration",
                user_name
            );
            Ok(())
        }
        None => {
            // User doesn't exist - this is good, they can register
            println!("User '{}' not found - can register", user_name);
            Ok(())
        }
    }
}

// Create user and store passkey atomically
pub async fn create_user_with_passkey(
    user_name: &str,
    passkey: &Passkey,
    state: &AppState,
) -> Result<(), AppError> {
    let user_id = Uuid::new_v4();

    let cred_id_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(passkey.cred_id().as_ref());

    println!(
        "Creating new user '{}' with ID: {} and passkey",
        user_name, user_id
    );

    // Insert user with passkey in one operation
    sqlx::query("INSERT INTO users (id, username, credential_id, passkey) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(user_name)
        .bind(&cred_id_b64)
        .bind(serde_json::to_vec(&passkey).map_err(|e| {
            eprintln!("Failed to serialize passkey: {}", e);
            AppError::SerializationError(e.to_string())
        })?)
        .execute(&state.db.pool)
        .await
        .map_err(|e| {
            eprintln!("Database error creating user with passkey: {}", e);
            AppError::DatabaseError(e.to_string())
        })?;

    println!("✅ User and passkey created successfully");
    Ok(())
}

// Authentication helpers
pub async fn get_user_passkeys(user_name: &str, state: &AppState) -> Result<Passkey, AppError> {
    //First we check if user exists
    let result = sqlx::query(
        r#"
        SELECT passkey
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(user_name)
    .fetch_optional(&state.db.pool)
    .await
    .map_err(|e| {
        eprintln!("Database error fetching user passkeys: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    match result {
        Some(row) => {
            let passkey_binary: Vec<u8> = row.try_get("passkey").map_err(|e| {
                eprintln!("Error extracting passkey from row: {}", e);
                AppError::DatabaseError(e.to_string())
            })?;

            //Deserialize passkey using serde_json
            let passkey: Passkey = serde_json::from_slice(&passkey_binary).map_err(|e| {
                eprintln!("Error deserializing passkey:{}", e);
                AppError::SerializationError(e.to_string())
            })?;

            println!("Retrieved passkey for user: {}", user_name);
            Ok(passkey)
        }
        None => {
            println!("User {} not found ", user_name);
            Err(AppError::UserNotFound)
        }
    }
}

pub async fn update_passkey_counter(
    user_name: &str,
    auth_result: &AuthenticationResult,
    state: &AppState,
) -> Result<(Uuid, String), AppError> {
    // lets get the pass by using the upper helper function
    let mut passkey = get_user_passkeys(user_name, state).await?;

    //lets update  the passkey counter now
    passkey.update_credential(auth_result);

    // Now we need to save the updated passkey back to the database and return user_id
    let updated_passkey_binary = serde_json::to_vec(&passkey).map_err(|e| {
        eprintln!("Error serializing updated passkey:{}", e);
        AppError::SerializationError(e.to_string())
    })?;

    let user = sqlx::query!(
        r#"
        UPDATE users
        SET passkey = $1
        WHERE username = $2
        RETURNING id, username
        "#,
        updated_passkey_binary,
        user_name
    )
    .fetch_one(&state.db.pool)
    .await
    .map_err(|e| {
        eprintln!("Database error updating passkey counter: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    println!(
        "Updated passkey counter for user: {} (ID: {})",
        user.username, user.id
    );
    Ok((user.id, user.username))
}
