use std::fmt::{Display, Formatter};
use askama::Template;
use axum::{Router};
use axum::routing::get;
use axum_core::response::IntoResponse;
use crate::AppData;
use crate::middleware::auth::{AuthSession};
use crate::ui::auth::LoginView;

mod auth;

pub fn ui() -> Router<AppData> {
    Router::new()
        .route("/", get(index))
        .nest("/auth", auth::route())
}

async fn index(auth: Option<AuthSession>) -> Page<LoginView<'static>> {
    match auth {
        Some(_) => unimplemented!(),
        None => Page(LoginView::default()),
    }
}

pub trait View: Display {
    fn title(&self) -> &'static str;
}

#[derive(Template)]
#[template(path = "page.html", escape = "none")]
pub struct Page<V: View>(pub V);
