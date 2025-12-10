use std::sync::Arc;
use std::time::Duration;
use sqlx::PgPool;
use webauthn_rs::prelude::*;
use moka::future::Cache;


pub type RegistrationStateCache = Cache<String, (PasskeyRegistration, String)>;
pub type AuthenticationStateCache = Cache<String, (PasskeyAuthentication, String)>;

#[derive(Clone)]
pub struct InMemoryStore {
    // Moka cache with automatic expiration (TTL: 5 minutes)
    pub registration_state: RegistrationStateCache,
    pub authentication_states: AuthenticationStateCache,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            // Auto-expires after 5 minutes, max 10k concurrent registrations
            registration_state: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(300)) // 5 minutes
                .build(),
            
            // Auto-expires after 5 minutes, max 10k concurrent authentications
            authentication_states: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_secs(300)) // 5 minutes
                .build(),
        }
    }
}

// ///////////////////////////////////
#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub webauth: Arc<Webauthn>,
    pub store: InMemoryStore,
    pub rp_id: String,
    pub db: PgPool,
}

impl AppState {
    pub fn new(webauth: Webauthn, rp_id: String, db: PgPool) -> Self {
        Self {
            webauth: Arc::new(webauth),
            store: InMemoryStore::new(),
            rp_id,
            db,
        }
    }
}