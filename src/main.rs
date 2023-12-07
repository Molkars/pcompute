extern crate core;

use anyhow::Context;
use axum::{Extension, Router};

mod util;
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
    let db = DB(sqlite_pool);

    let redis = redis::Client::open("redis://127.0.0.1/")
        .context("unable to create redis client")?;
    let redis = Redis(redis);

    let app = Router::new()
        .nest("/api", api::api())
        .layer(Extension(db))
        .layer(Extension(redis));

    Ok(())
}

fn envs() {
    let password_pepper = std::env::var("PASSWORD_PEPPER")
        .expect("PASSWORD_PEPPER must be set & mut be 48 bytes long");
    assert_eq!(password_pepper.len(), 48);
}


pub type DbPool = sqlx::SqlitePool;
#[derive(Clone)]
pub struct DB(DbPool);
#[derive(Clone)]
pub struct Redis(redis::Client);

pub struct AppData {
    pub db: DB,
    pub redis: Redis,
}