use axum::Router;

mod auth;

pub fn api() -> Router {
    Router::new()
}