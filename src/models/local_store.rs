
use std::{collections::HashMap, sync::Arc};
use sqlx::PgPool;
use tokio::sync::RwLock;
use webauthn_rs::prelude::*;

pub type SharedMap<K, V> = Arc<RwLock<HashMap<K, V>>>;


#[derive(Clone)]
pub struct InMemoryStore {
    // these are temporary states during registration and authentication
    pub registration_state: SharedMap<String, (PasskeyRegistration, String)>, // Store username instead of Uuid
    pub authentication_states: SharedMap<String, (PasskeyAuthentication,String)>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            registration_state: Arc::new(RwLock::new(HashMap::new())),
            authentication_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}


// ///////////////////////////////////
#[derive(Clone)]
pub struct AppState{
    pub webauth: Arc<Webauthn>,
    pub store: InMemoryStore,
    pub rp_id:String,
    pub db:PgPool,
}

impl AppState{
    pub fn new(webauth:Webauthn , rp_id:String, db:PgPool)->Self{
        Self{
            webauth:Arc::new(webauth),
            store:InMemoryStore::new(),
            rp_id,
            db,
        }
    }
}