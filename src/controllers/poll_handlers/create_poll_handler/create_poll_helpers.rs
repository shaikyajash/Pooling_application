use sqlx::{Postgres, Transaction, types::Uuid};

use crate::models::{
    polls::{Poll, PollOption},
};

/// Insert poll into database within a transaction
pub async fn insert_poll(
    tx: &mut Transaction<'_, Postgres>,
    poll_id: Uuid,
    title: &str,
    creator_id: Uuid,
) -> Result<Poll, sqlx::Error> {
    sqlx::query_as::<_, Poll>(
        r#"
        INSERT INTO polls (id, title, creator_id)
        VALUES ($1, $2, $3)
        RETURNING id, title, creator_id, is_closed, created_at, closed_at, total_votes
        "#,
    )
    .bind(poll_id)
    .bind(title.trim())
    .bind(creator_id)
    .fetch_one(&mut **tx)
    .await
}



/// Insert poll options into database within a transaction
pub async fn insert_poll_options(
    tx: &mut Transaction<'_, Postgres>,
    poll_id: Uuid,
    options: &[String],
) -> Result<Vec<PollOption>, sqlx::Error> {
    let mut poll_options = Vec::new();

    for (index, option_text) in options.iter().enumerate() {
        let option_id = Uuid::new_v4();
        let option = sqlx::query_as::<_, PollOption>(
            r#"
            INSERT INTO poll_options (id, poll_id, option_text, display_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, poll_id, option_text, vote_count, display_order
            "#,
        )
        .bind(option_id)
        .bind(poll_id)
        .bind(option_text.trim())
        .bind(index as i32)
        .fetch_one(&mut **tx)
        .await?;

        poll_options.push(option);
    }

    Ok(poll_options)
}