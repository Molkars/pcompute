use anyhow::Context;

mod auth;

#[derive(sqlx::FromRow)]
pub struct RawUser {
    pub id: i64,
    pub username: String,
    pub password: String,
}

pub struct User {
    pub id: i64,
    pub username: String,
}

pub struct UserController {
    db: crate::DB,
}

impl UserController {
    pub fn new(db: crate::DB) -> Self {
        Self {
            db
        }
    }

    pub async fn get_user_by_username<T: From<RawUser>>(&self, username: impl AsRef<str>) -> anyhow::Result<T> {
        let username = username.as_ref();
        let user = sqlx::query_as!(RawUser,
            r#"
            SELECT id, username, password
            FROM users
            WHERE username = ?
            "#,
            username
        )
            .fetch_one(&self.db.0)
            .await?;

        Ok(T::from(user))
    }

    pub async fn get_user_by_id<T: From<RawUser>>(&self, id: i64) -> anyhow::Result<T> {
        let user = sqlx::query_as!(RawUser,
            r#"
            SELECT id, username, password
            FROM users
            WHERE id = ?
            "#,
            id
        )
            .fetch_one(&self.db.0)
            .await?;

        Ok(T::from(user))
    }

    pub async fn update_user(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE users
            SET username = ?
            WHERE id = ? AND username = ?
            "#,
            user.username, user.id, user.username
        )
            .execute(&self.db.0)
            .await?;

        Ok(())
    }

    pub async fn update_user_password(&self,
                                      user_id: i64,
                                      username: &str,
                                      password: impl AsRef<str>
    ) -> anyhow::Result<()> {
        let password = password.as_ref();
        let password = auth::hash_password(password)
            .context("Failed to hash password")?;
        sqlx::query!(
            r#"
            UPDATE users
            SET password = ?
            WHERE id = ? AND username = ?
            "#,
            password, user_id, username
        )
            .execute(&self.db.0)
            .await?;

        Ok(())
    }

    pub async fn create_user<T: From<RawUser>>(&self, username: impl AsRef<str>, password: impl AsRef<str>) -> anyhow::Result<T> {
        let username = username.as_ref();
        let password = password.as_ref();
        let password = auth::hash_password(password)
            .context("Failed to hash password")?;
        let user_id = sqlx::query!(
            r#"
            INSERT INTO users (username, password)
            VALUES (?, ?)
            "#,
            username, password
        )
            .execute(&self.db.0)
            .await?
            .last_insert_rowid();

        self.get_user_by_id::<T>(user_id).await
    }

    pub async fn validate_credentials<T: From<RawUser>>(&self, username: impl AsRef<str>, password: impl AsRef<str>) -> anyhow::Result<Option<T>> {
        let username = username.as_ref();
        let password = password.as_ref();
        let user = self.get_user_by_username::<RawUser>(username).await?;
        let valid = auth::verify_password(password, &user.password)
            .context("Failed to verify password")?;
        Ok(if valid {
            Some(T::from(user))
        } else {
            None
        })
    }
}

impl From<RawUser> for User {
    fn from(value: RawUser) -> Self {
        Self {
            id: value.id,
            username: value.username,
        }
    }
}

impl From<RawUser> for () {
    fn from(_: RawUser) -> Self {}
}