use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;




#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Poll {
    pub id: Uuid,
    pub title: String,
    pub creator_id: Uuid,
    pub is_closed: bool,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub total_votes: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct PollOption {
    pub id: Uuid,
    pub poll_id: Uuid,
    pub option_text: String,
    pub vote_count: i32,
    pub display_order: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PollOptionWithPercentage {
    #[serde(flatten)]
    pub option: PollOption,
    pub percentage: f64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct PollWithOptions {
    #[serde(flatten)]
    pub poll: Poll,
    pub options: Vec<PollOptionWithPercentage>,
    pub user_voted_option_id: Option<Uuid>, // Which option the user voted for (if any)
}
