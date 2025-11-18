use sqlx::{PgPool, Pool, Postgres, migrate::MigrateDatabase};
use std::env;

pub type DbPool = Pool<Postgres>;

pub async fn establish_connection() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    PgPool::connect(&database_url).await
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    // Run migrations from the migrations directory
    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations")).await?;
    migrator.run(pool).await?;
    Ok(())
}