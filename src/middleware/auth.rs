use async_trait::async_trait;
use axum::extract::State;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_core::extract::FromRef;
use axum_core::RequestPartsExt;
use axum_extra::extract::CookieJar;
use crate::controller::session::{Session, SessionController};
use crate::Redis;

pub struct AuthSession(pub Session);

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for AuthSession
    where S: Send + Sync,
          Redis: FromRef<S>
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookie_jar: CookieJar = req.extract::<CookieJar>().await.unwrap();
        if let Some(session_cookie) = cookie_jar.get(SessionController::SESSION_COOKIE_NAME) {
            let session_id = session_cookie.value();

            let State(redis) = State::<Redis>::from_request_parts(req, state).await
                .expect("Redis state not found");
            let session_controller = SessionController::new(redis.clone());
            let session = session_controller.get_session(session_id).await
                .map_err(|e| {
                    eprintln!("Error getting session: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
                })?;

            match session {
                Some(session) => Ok(Self(session)),
                None => Err((StatusCode::UNAUTHORIZED, "unauthorized")),
            }
        } else {
            Err((StatusCode::UNAUTHORIZED, "unauthorized"))
        }
    }
}
