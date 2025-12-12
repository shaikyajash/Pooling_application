use sqlx::{types::Uuid};

use crate::models::{
    local_store::AppState,
    polls::{Poll, PollOption},
};


/// Get poll by ID
pub async fn get_poll_by_id(poll_id: &Uuid, state: &AppState) -> Result<Poll, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        WHERE id = $1
        "#,
    )
    .bind(poll_id)
    .fetch_one(&state.db)
    .await
}

/// Get poll options by poll ID
pub async fn get_poll_options(
    poll_id: &Uuid,
    state: &AppState,
) -> Result<Vec<PollOption>, sqlx::Error> {
    sqlx::query_as::<_, PollOption>(
        r#"
        SELECT id, poll_id, option_text, vote_count, display_order
        FROM poll_options
        WHERE poll_id = $1
        ORDER BY display_order
        "#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await
}

/// Get user's vote for a poll (returns option_id if voted)
pub async fn get_user_vote(
    poll_id: &Uuid,
    user_id: &Uuid,
    state: &AppState,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT option_id
        FROM votes
        WHERE poll_id = $1 AND user_id = $2
        "#,
    )
    .bind(poll_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    Ok(result.map(|r| r.0))
}



/// Close a poll
pub async fn close_poll(poll_id: &Uuid, state: &AppState) -> Result<Poll, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        UPDATE polls
        SET is_closed = true, closed_at = NOW()
        WHERE id = $1
        RETURNING id, title, creator_id, is_closed, created_at, closed_at, total_votes
        "#,
    )
    .bind(poll_id)
    .fetch_one(&state.db)
    .await
}



/// Reset poll votes
pub async fn reset_poll_votes(poll_id: &Uuid, state: &AppState) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;

    // Delete all votes for this poll
    sqlx::query(
        r#"
        DELETE FROM votes
        WHERE poll_id = $1
        "#,
    )
    .bind(poll_id)
    .execute(&mut *tx)
    .await?;

    // Reset vote counts for all options
    sqlx::query(
        r#"
        UPDATE poll_options
        SET vote_count = 0
        WHERE poll_id = $1
        "#,
    )
    .bind(poll_id)
    .execute(&mut *tx)
    .await?;

    // Reset total votes for the poll
    sqlx::query(
        r#"
        UPDATE polls
        SET total_votes = 0
        WHERE id = $1
        "#,
    )
    .bind(poll_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}




/// Cast a vote (transaction-safe) and return the voted option
pub async fn cast_vote(
    poll_id: &Uuid,
    user_id: &Uuid,
    option_id: &Uuid,
    state: &AppState,
) -> Result<PollOption, sqlx::Error> {
    let mut tx = state.db.begin().await?;

    // Insert the vote
    let vote_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO votes (id, poll_id, user_id, option_id)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(vote_id)
    .bind(poll_id)
    .bind(user_id)
    .bind(option_id)
    .execute(&mut *tx)
    .await?;

    // Increment the option vote count and return the updated option
    let voted_option = sqlx::query_as::<_, PollOption>(
        r#"
        UPDATE poll_options
        SET vote_count = vote_count + 1
        WHERE id = $1
        RETURNING id, poll_id, option_text, vote_count, display_order
        "#,
    )
    .bind(option_id)
    .fetch_one(&mut *tx)
    .await?;

    // Increment the poll total votes
    sqlx::query(
        r#"
        UPDATE polls
        SET total_votes = total_votes + 1
        WHERE id = $1
        "#,
    )
    .bind(poll_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(voted_option)
}





/// Get poll with all options in a single query (optimized for SSE)
pub async fn get_poll_with_options(
    poll_id: &Uuid,
    state: &AppState,
) -> Result<(Poll, Vec<PollOption>), sqlx::Error> {
    // Fetch poll first
    let poll = sqlx::query_as::<_, Poll>(
        r#"
        SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        WHERE id = $1
        "#,
    )
    .bind(poll_id)
    .fetch_one(&state.db)
    .await?;

    // Fetch options in same connection
    let options = sqlx::query_as::<_, PollOption>(
        r#"
        SELECT id, poll_id, option_text, vote_count, display_order
        FROM poll_options
        WHERE poll_id = $1
        ORDER BY display_order
        "#,
    )
    .bind(poll_id)
    .fetch_all(&state.db)
    .await?;

    Ok((poll, options))
}




// New: fetch all polls (summary)
pub async fn get_all_polls(state: &AppState) -> Result<Vec<Poll>, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
}
