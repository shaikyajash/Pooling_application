use sqlx::types::Uuid;

use crate::models::{local_store::AppState, polls::Poll};


/// Get all polls created by a specific user (summary, no options)
pub async fn get_polls_by_user(
    user_id: &Uuid,
    state: &AppState,
) -> Result<Vec<Poll>, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        WHERE creator_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}