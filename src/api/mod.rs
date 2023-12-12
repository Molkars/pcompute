use axum::Router;
use axum::routing::get;
use crate::AppData;

mod auth;

pub fn api() -> Router<AppData> {
    Router::new()
        .route("/", get(index))
}

async fn index() -> &'static str {
    "Hello, World!"
}

pub struct ApiError {
    inner: anyhow::Error,
}

impl ApiError {
    pub fn new(inner: anyhow::Error) -> Self {
        Self { inner }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(inner: anyhow::Error) -> Self {
        Self::new(inner)
    }
}