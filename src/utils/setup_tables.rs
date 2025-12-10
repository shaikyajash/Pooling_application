

pub async fn make_tables_if_not_exists(pool: &sqlx::PgPool)->Result<(), sqlx::Error>{
    // Create users table - 1 username -> 1 passkey (simplified)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            credential_id VARCHAR(255) UNIQUE,
            passkey BYTEA
        )
        "#
    )
    .execute(pool)
    .await?;

    println!("✅ Database tables setup complete");
    Ok(())
}