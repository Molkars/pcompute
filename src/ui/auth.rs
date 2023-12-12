use std::borrow::Cow::{self, Borrowed};
use askama::Template;
use axum::{Form, Router};
use axum::extract::State;
use axum::routing::post;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use derive_setters::Setters;
use log::{error, debug};
use serde_derive::{Deserialize};
use crate::{AppData, DB, Redis};
use crate::controller::session::SessionController;
use crate::controller::user::{User, UserController};
use crate::middleware::auth::AuthSession;
use crate::ui::View;

pub fn route() -> Router<AppData> {
    Router::new()
        .route("/login", post(login))
}

pub async fn login(
    State(db): State<DB>,
    State(redis): State<Redis>,
    auth_session: Option<AuthSession>,
    Form(form): Form<LoginForm>,
) -> (CookieJar, LoginView<'static>) {
    if auth_session.is_none() {
        let user_controller = UserController::new(db.clone());

        match user_controller.validate_credentials::<User>(form.username.as_str(), form.password.as_str()).await {
            Ok(Some(user)) => {
                debug!(target: "auth", "User {} logged in", user.username);
                let session_controller = SessionController::new(redis.clone());
                let session = session_controller.create_session(user.id).await.unwrap();

                let cookie_jar = CookieJar::default()
                    .add(Cookie::new(SessionController::SESSION_COOKIE_NAME, session.session_id().to_string()));
                (cookie_jar, Box::new(LoginView::default())
            }
            Ok(None) => {
                debug!(target: "auth", "Invalid credentials for user {}", form.username);
                (CookieJar::default(), LoginView::default().error(Borrowed("Invalid credentials")))
            }
            Err(e) => {
                error!(target: "auth", "Failed to validate credentials: {}", e);
                (CookieJar::default(), LoginView::default().error(Borrowed("Failed to validate credentials")))
            }
        }
    } else {
        debug!(target: "auth", "Already logged in: {:?}", form);
        (CookieJar::default(), LoginView::default().error(Borrowed("Already logged in")))
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Template, Default, Setters)]
#[template(path = "auth/login.html")]
pub struct LoginView<'a> {
    username: Cow<'a, str>,
    error: Cow<'a, str>,
}

impl View for LoginView<'_> {
    fn title(&self) -> &'static str {
        "Login"
    }
}

#[test]
fn test_page() {
    let rendered = LoginView::default()
        .username(Borrowed("test"))
        .error(Borrowed("error"))
        .render()
        .unwrap();
    println!("{}", rendered);
}