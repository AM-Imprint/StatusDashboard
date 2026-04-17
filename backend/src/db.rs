use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(database_url)
        .expect("Invalid database URL")
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(options)
        .await
        .expect("Failed to connect to SQLite database")
}

pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run database migrations");
}
