use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    user_id: i64,
    session_id: String,
    session_key: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    pub fn session_key(&self) -> &str {
        &self.session_key
    }
    pub fn expires_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.expires_at
    }
}

pub struct SessionController {
    redis: crate::Redis,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Session not found")]
    NotFound,
    #[error("Failed to get session")]
    Redis(#[from] redis::RedisError),
    #[error("Failed to encode session")]
    Json(#[from] serde_json::Error),
}

impl SessionController {
    pub fn new(redis: crate::Redis) -> Self {
        Self {
            redis
        }
    }

    pub async fn create_session(&self, user_id: i64) -> Result<Session, Error> {
        let session = Session {
            user_id,
            session_id: crate::util::rand::random_string_with_prefix(32, "session"),
            session_key: crate::util::rand::random_string_with_prefix(32, "sk"),
            expires_at: chrono::Utc::now() + chrono::Duration::days(1),
        };

        let session_json = crate::util::convert::json_encode(&session)?;

        let mut conn = self.redis.0.get_async_connection().await?;
        redis::cmd("SET")
            .arg(format!("session:{}", session.session_id))
            .arg(&session_json)
            .arg("PXAT")
            .arg(session.expires_at.timestamp_millis())
            .query_async(&mut conn)
            .await?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Session, Error> {
        let mut conn = self.redis.0.get_async_connection().await?;
        let session_json: Option<String> = redis::cmd("GET")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await?;

        let Some(session_json) = session_json else {
            return Err(Error::NotFound);
        };

        let session: Session = crate::util::convert::json_decode(&session_json)?;

        if session.expires_at < chrono::Utc::now() {
            let _: () = redis::cmd("DEL")
                .arg(format!("session:{}", session_id))
                .query_async(&mut conn)
                .await?;
            return Err(Error::NotFound);
        }

        Ok(session)
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.redis.0.get_async_connection().await?;
        let _: () = redis::cmd("DEL")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}
