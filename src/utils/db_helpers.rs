use sqlx::{PgPool, types::Uuid};

use crate::{
    models::polls::{Poll, PollOption},
    utils::connect_to_db::connect_to_db,
};

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = match connect_to_db().await {
            Ok(pool) => pool,
            Err(e) => return Err(e),
        };

        Ok(Self { pool })
    }

    pub async fn insert_poll_with_options(
        &self,
        poll_id: Uuid,
        title: &str,
        creator_id: Uuid,
        options: &[String],
    ) -> Result<(Poll, Vec<PollOption>), sqlx::Error> {
        // Start transaction
        let mut tx = self.pool.begin().await?;

        // 1. Insert the poll
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            INSERT INTO polls (id, title, creator_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, creator_id, is_closed, created_at, closed_at, total_votes
            "#,
        )
        .bind(poll_id)
        .bind(title.trim())
        .bind(creator_id)
        .fetch_one(&mut *tx)
        .await?;

        // 2. Insert all poll options
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
            .fetch_one(&mut *tx)
            .await?;

            poll_options.push(option);
        }

        // 3. Commit the transaction
        tx.commit().await?;

        Ok((poll, poll_options))
    }

    pub async fn get_polls_by_user_id(&self, user_id: &Uuid) -> Result<Vec<Poll>, sqlx::Error> {
        sqlx::query_as::<_, Poll>(
            r#" SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        WHERE creator_id = $1
        ORDER BY created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_poll_by_id(&self, poll_id: &Uuid) -> Result<Poll, sqlx::Error> {
        sqlx::query_as::<_, Poll>(
            r#"
        SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
        FROM polls
        WHERE id = $1
        "#,
        )
        .bind(poll_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Get poll options by poll ID
    pub async fn get_poll_options(&self, poll_id: &Uuid) -> Result<Vec<PollOption>, sqlx::Error> {
        sqlx::query_as::<_, PollOption>(
            r#"
        SELECT id, poll_id, option_text, vote_count, display_order
        FROM poll_options
        WHERE poll_id = $1
        ORDER BY display_order
        "#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Get user's vote for a poll (returns option_id if voted)
    pub async fn get_user_vote(
        &self,
        poll_id: &Uuid,
        user_id: &Uuid,
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
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some(r) => Ok(Some(r.0)),
            None => Ok(None),
        }
    }
    /// Close a poll
    pub async fn close_poll(&self, poll_id: &Uuid) -> Result<Poll, sqlx::Error> {
        sqlx::query_as::<_, Poll>(
            r#"
        UPDATE polls
        SET is_closed = true, closed_at = NOW()
        WHERE id = $1
        RETURNING id, title, creator_id, is_closed, created_at, closed_at, total_votes
        "#,
        )
        .bind(poll_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Reset poll votes
    pub async fn reset_poll_votes(&self, poll_id: &Uuid) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        //Delete all votes for this poll

        sqlx::query(
            r#"
        DELETE FROM votes
        WHERE poll_id = $1
        "#,
        )
        .bind(poll_id)
        .execute(&mut *tx)
        .await?;

        //Reset vote counts for all options
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

        //Reset total votes for the poll
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

    pub async fn cast_vote(
        &self,
        poll_id: &Uuid,
        user_id: &Uuid,
        option_id: &Uuid,
    ) -> Result<PollOption, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        //Insert the vote
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

        //Increment the option vote count and return the updated option
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

        //Increment the poll total votes
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
        &self,
        poll_id: &Uuid,
    ) -> Result<(Poll, Vec<PollOption>), sqlx::Error> {
        // Fetch poll first and then fetch options in same connection
        let poll = sqlx::query_as::<_, Poll>(
            r#"
            SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
            FROM polls
            WHERE id = $1
            "#,
        )
        .bind(poll_id)
        .fetch_one(&self.pool)
        .await?;

        let options = sqlx::query_as::<_, PollOption>(
            r#"
            SELECT id, poll_id, option_text, vote_count, display_order
            FROM poll_options
            WHERE poll_id = $1
            ORDER BY display_order
            "#,
        )
        .bind(poll_id)
        .fetch_all(&self.pool)
        .await?;

        Ok((poll, options))
    }

    ///Fetch all polls(Summary)
    pub async fn get_all_polls(&self) -> Result<Vec<Poll>, sqlx::Error> {
        sqlx::query_as::<_, Poll>(
            r#"
            SELECT id, title, creator_id, is_closed, created_at, closed_at, total_votes
            FROM polls
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}
