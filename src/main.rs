extern crate core;

use anyhow::Context;
use axum::{Router};
use axum_core::extract::FromRef;
use log::LevelFilter;

mod util;
mod api;
mod ui;
pub(crate) mod middleware;
pub(crate) mod controller;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .filter_module("sqlx", LevelFilter::Info)
        .init();
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
        .nest("/", ui::ui())
        .with_state(AppData { db: db.clone(), redis: redis.clone() });

    let stream = tokio::net::TcpListener::bind("0.0.0.0:8192").await?;
    axum::serve(stream, app).await?;

    Ok(())
}

fn envs() {
    std::env::set_var("PASSWORD_PEPPER", "abcdef".repeat(8));
    let password_pepper = std::env::var("PASSWORD_PEPPER")
        .expect("PASSWORD_PEPPER must be set & mut be 48 bytes long");
    assert_eq!(password_pepper.len(), 48);
}


pub type DbPool = sqlx::SqlitePool;
#[derive(Clone)]
pub struct DB(DbPool);
#[derive(Clone)]
pub struct Redis(redis::Client);

#[derive(Clone)]
pub struct AppData {
    pub db: DB,
    pub redis: Redis,
}

impl FromRef<AppData> for DB {
    fn from_ref(input: &AppData) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppData> for Redis {
    fn from_ref(input: &AppData) -> Self {
        input.redis.clone()
    }
}