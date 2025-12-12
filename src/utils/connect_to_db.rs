use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect_to_db() -> Result<PgPool, sqlx::Error> {
    // let database_url = std::env::var("DATABASE_URL")
    //     .map_err(|_| sqlx::Error::Configuration("DATABASE_URL not set".into()))?;

    let database_url = match std::env::var("DATABASE_URL"){
        Ok(url)=>url,
        Err(_) =>{
            eprintln!("DATABASE_URL not set in .env file");
            return Err(sqlx::Error::Configuration("DATABASE_URL not set".into()));
        }
    };

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => {
            println!("Successfully connected to the database.");
            pool
        }
        Err(e) => {
            eprintln!(
                "Make sure your .env file is configured correctly and the DATABASE_URL is set."
            );
            return Err(e);
        }
    };

    Ok(pool)
}
