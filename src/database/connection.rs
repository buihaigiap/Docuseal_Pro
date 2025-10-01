use sqlx::{PgPool, Pool, Postgres};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn establish_connection() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPool::connect(&database_url).await
}