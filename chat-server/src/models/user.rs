use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chat_core::models::user::CurUser;
use serde::{Deserialize, Serialize};
use sqlx::{
    prelude::FromRow,
    types::chrono::{DateTime, Utc},
    MySql, Pool,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, FromRow)]
pub(crate) struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    #[sqlx(Default)]
    #[serde(skip)]
    pub password: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for CurUser {
    fn from(value: User) -> Self {
        CurUser {
            id: value.id,
            ws_id: value.ws_id,
            fullname: value.fullname,
            email: value.email,
        }
    }
}

impl User {
    // 创建用户
    pub(crate) async fn create(
        ws_id: i64,
        fullname: String,
        password: String,
        email: String,
        pool: &Pool<MySql>,
    ) -> std::result::Result<i64, sqlx::Error> {
        let res =
            sqlx::query("insert into users (ws_id, fullname, password, email) values (?, ?, ?, ?)")
                .bind(ws_id)
                .bind(fullname)
                .bind(password)
                .bind(email)
                .execute(pool)
                .await?;

        Ok(res.last_insert_id() as i64)
    }

    // 根据邮箱精确查询一个用户
    pub(crate) async fn find_by_email(email: &String, pool: &Pool<MySql>) -> Result<Option<User>> {
        let res = sqlx::query_as::<_, User>("select * from users where email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(res)
    }

    // 根据用户ID精确查询一个用户
    pub(crate) async fn find_by_id(id: i64, pool: &Pool<MySql>) -> Result<Option<User>> {
        let res = sqlx::query_as::<_, User>("select * from users where id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(res)
    }

    // 使用argon2算法hash原始密码
    pub(crate) fn hash_password(password: &String) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();

        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        Ok(password_hash)
    }
    // 使用argon2算法验证密码
    pub(crate) fn verify_password(password: &String, password_hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(password_hash)?;
        let argon2 = Argon2::default();
        // Verify password against PHC string
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();

        Ok(is_valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_db_tester::TestMySql;
    use std::path::Path;

    #[tokio::test]
    async fn test_find_by_email() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let _user_id = User::create(
            1,
            "test".to_owned(),
            "password".to_owned(),
            "test@test.com".to_owned(),
            &pool,
        )
        .await
        .unwrap();

        let user = User::find_by_email(&"test@test.com".to_owned(), &pool)
            .await
            .unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, "test@test.com");
    }

    #[tokio::test]
    async fn password_hash_verify_test() {
        let password = "password".to_owned();
        let password_hash = User::hash_password(&password).unwrap();
        assert!(User::verify_password(&password, &password_hash).unwrap());
    }
}
