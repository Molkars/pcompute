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
    pub db: crate::DB,
}

impl UserController {
    pub async fn get_user_by_username<T: From<RawUser>>(&self, username: impl AsRef<str>) -> anyhow::Result<T> {
        let username = username.as_ref();
        let user = sqlx::query_as::<_, RawUser>(
            r#"
            SELECT id, username, password
            FROM users
            WHERE username = ?
            "#,
        )
            .bind(&username)
            .fetch_one(&self.db.0)
            .await?;

        Ok(T::from(user))
    }

    pub async fn update_user(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET username = ?
            WHERE id = ? AND username = ?
            "#,
        )
            .bind(&user.username)
            .bind(&user.id)
            .bind(&user.username)
            .execute(&self.db.0)
            .await?;

        Ok(())
    }

    pub async fn update_user_password(&self,
                                      user_id: i64,
                                      username: &str,
                                      password: impl AsRef<str>,
                                      current_password: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let password = password.as_ref();
        let password = auth::hash_password(password)
            .context("Failed to hash password")?;
        sqlx::query(
            r#"
            UPDATE users
            SET password = ?
            WHERE id = ? AND username = ?
            "#,
        )
            .bind(&password)
            .bind(&user_id)
            .bind(&username)
            .execute(&self.db.0)
            .await?;

        Ok(())
    }
}