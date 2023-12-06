use anyhow::Context;

mod api;
pub(crate) mod controller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    envs();

    let sqlite_pool = sqlx::sqlite::SqlitePool::connect("sqlite:./db.sqlite3?mode=rwc")
        .await
        .context("Failed to connect to SQLite")?;
    sqlx::migrate!()
        .run(&sqlite_pool)
        .await
        .context("Failed to run migrations")?;

    println!("Hello, world!");

    Ok(())
}

fn envs() {
    let password_pepper = std::env::var("PASSWORD_PEPPER")
        .expect("PASSWORD_PEPPER must be set & mut be 48 bytes long");
    assert_eq!(password_pepper.len(), 48);
}


pub type DbPool = sqlx::SqlitePool;
pub struct DB(DbPool);