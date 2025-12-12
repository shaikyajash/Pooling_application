

pub async fn make_tables_if_not_exists(pool: &sqlx::PgPool)->Result<(), sqlx::Error>{
    // Create users table - 1 username -> 1 passkey (simplified)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            credential_id VARCHAR(255) UNIQUE,
            passkey BYTEA,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(pool)
    .await?;

      // Create polls table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS polls (
            id UUID PRIMARY KEY,
            title VARCHAR(500) NOT NULL,
            creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            is_closed BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            closed_at TIMESTAMP WITH TIME ZONE,
            total_votes INTEGER DEFAULT 0
        )
        "#
    )
    .execute(pool)
    .await?;


    // Create poll_options table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS poll_options (
            id UUID PRIMARY KEY,
            poll_id UUID NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
            option_text VARCHAR(255) NOT NULL,
            vote_count INTEGER DEFAULT 0,
            display_order INTEGER NOT NULL,
            UNIQUE(poll_id, option_text)
        )
        "#
    )
    .execute(pool)
    .await?;


// Create votes table (to track who voted on what)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS votes (
            id UUID PRIMARY KEY,
            poll_id UUID NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
            user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            option_id UUID NOT NULL REFERENCES poll_options(id) ON DELETE CASCADE,
            voted_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(poll_id, user_id)
        )
        "#
    )
    .execute(pool)
    .await?;





    // Create indexes for better query performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_polls_creator ON polls(creator_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_poll_options_poll ON poll_options(poll_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_votes_poll ON votes(poll_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_votes_user ON votes(user_id)")
        .execute(pool)
        .await?;





    println!("✅ Database tables setup complete");
    Ok(())
}