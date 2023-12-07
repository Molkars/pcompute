use std::task::{Context, Poll};
use axum::{async_trait, Extension, RequestExt};
use axum::extract::Request;
use axum::http::{StatusCode};
use axum::http::request::Parts;
use axum::response::Response;
use axum_core::body::Body;
use futures::future::BoxFuture;
use tower::{Layer, Service};
use crate::controller::session::Session;
use crate::Redis;

#[derive(Debug, Clone)]
pub struct Authenticator;

impl<S> Layer<S> for Authenticator {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthService<S> {
    inner: S,
}

#[derive(Clone)]
pub struct AuthSession(pub Session);


impl<S> Service<Request> for AuthService<S>
    where
        S: Service<Request, Response=Response> + Clone + Send + 'static,
        S::Future: Send + 'static
{
    type Response = Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            let session_id = {
                let Some(session_id) = req.headers().get("X-Grey-Session-Id") else {
                    return inner.call(req).await;
                };

                // todo: better error messages
                let Ok(value) = session_id.to_str() else {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Default::default())
                        .unwrap());
                };

                let Some(session_id) = value.strip_prefix("Bearer ") else {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Default::default())
                        .unwrap());
                };

                session_id.to_string()
            };

            let Extension(redis): Extension<Redis> = req.extract_parts()
                .await
                .expect("Redis extension is missing");

            let controller = crate::controller::session::SessionController::new(redis);

            let Some(session) = controller.get_session(&session_id).await.ok() else {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Default::default())
                    .unwrap());
            };

            let Some(value) = req.headers().get("X-Grey-Session-Key") else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Default::default())
                    .unwrap());
            };

            let Ok(session_key) = value.to_str() else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Default::default())
                    .unwrap());
            };

            if session.session_key() != session_key {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Default::default())
                    .unwrap());
            }

            if session.expires_at() < chrono::Utc::now() {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Default::default())
                    .unwrap());
            }

            req.extensions_mut().insert(AuthSession(session));

            inner.call(req).await
        })
    }
}

pub struct Auth(pub Session);

pub struct DenyAuth;

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for Auth {
    type Rejection = Response<Body>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if let Some(session) = parts.extensions.get::<AuthSession>() {
            return Ok(Auth(session.0.clone()));
        } else {
            return Err(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Default::default())
                .unwrap());
        }
    }
}

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for DenyAuth {
    type Rejection = Response<Body>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        if parts.extensions.get::<AuthSession>().is_some() {
            Err(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body("This resource is forbidden for session users".into())
                .unwrap())
        } else {
            Ok(DenyAuth)
        }
    }
}