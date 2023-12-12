use axum::extract::State;
use axum::http::StatusCode;
use axum::{Form, Json};
use log::error;
use serde_json as json;
use serde_json::json;
use crate::{DB, Redis};
use crate::controller::session::{SessionController};
use crate::controller::user::{User, UserController};

pub async fn create_user(
    State(redis): State<Redis>,
    State(db): State<DB>,
    req: Form<CreateUserRequest>,
) -> (StatusCode, Json<json::Value>) {
    let user_controller = UserController::new(db);
    let session_controller = SessionController::new(redis);

    let user = user_controller.create_user(req.username.as_str(), req.password.as_str()).await;
    let user: User = match user {
        Ok(user) => user,
        Err(e) => {
            error!(target: "auth", "Failed to create user: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "an internal error occurred",
            })));
        }
    };

    let session = match session_controller.create_session(user.id).await {
        Ok(session) => session,
        Err(e) => {
            error!(target: "auth", "Failed to create session: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": "an internal error occurred",
            })));
        }
    };

    todo!()
}

pub struct CreateUserRequest {
    username: String,
    password: String,
}