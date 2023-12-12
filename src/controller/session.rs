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

impl SessionController {
    pub const SESSION_COOKIE_NAME: &'static str = "Grey-Session-Id";

    pub fn new(redis: crate::Redis) -> Self {
        Self {
            redis
        }
    }

    pub async fn create_session(&self, user_id: i64) -> anyhow::Result<Session> {
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

    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let mut conn = self.redis.0.get_async_connection().await?;
        let session_json: Option<String> = redis::cmd("GET")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await?;

        match session_json {
            None => Ok(None),
            Some(session_json) => {
                let session: Session = crate::util::convert::json_decode(&session_json)?;
                if session.expires_at < chrono::Utc::now() {
                    redis::cmd("DEL")
                        .arg(format!("session:{}", session_id))
                        .query_async(&mut conn)
                        .await?;
                    Ok(None)
                } else {
                    Ok(Some(session))
                }
            }
        }
    }

    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut conn = self.redis.0.get_async_connection().await?;
        redis::cmd("DEL")
            .arg(format!("session:{}", session_id))
            .query_async(&mut conn)
            .await?;

        Ok(())
    }
}
