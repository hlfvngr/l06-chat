use sqlx::{error::ErrorKind, MySql, Pool};

use crate::{error::AppError, models::user::User};

#[derive(Debug)]
pub(crate) struct UserService {
    pub(crate) pool: Pool<MySql>,
}

impl UserService {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }

    // 根据邮箱查询用户
    pub async fn find_by_email(&self, email: &String) -> Result<Option<User>, AppError> {
        Ok(User::find_by_email(email, &self.pool).await?)
    }

    // 根据用户ID查询用户
    pub async fn find_by_id(&self, user_id: i64) -> Result<Option<User>, AppError> {
        Ok(User::find_by_id(user_id, &self.pool).await?)
    }

    // 创建用户
    pub async fn create(
        &self,
        ws_id: i64,
        fullname: String,
        password: String,
        email: String,
    ) -> Result<i64, AppError> {
        // 根据邮箱查询用户，如果存在则返回邮箱已存在
        if User::find_by_email(&email, &self.pool).await?.is_some() {
            return Err(AppError::EmailAlreadyExists);
        }
        // 对传入的密码进行hash
        let password_hash = User::hash_password(&password)?;
        // 创建用户
        let user_id = match User::create(ws_id, fullname, password_hash, email, &self.pool).await {
            Ok(user_id) => user_id,
            Err(e) => {
                if let Some(err) = e.as_database_error() {
                    if ErrorKind::UniqueViolation == err.kind() {
                        return Err(AppError::EmailAlreadyExists);
                    }
                }
                return Err(e.into());
            }
        };
        Ok(user_id)
    }

    // 校验用户ID列表的合法性
    pub async fn validate_user_ids(&self, user_ids: &Vec<i64>) -> Result<(), AppError> {
        for user_id in user_ids {
            let user = User::find_by_id(*user_id, &self.pool).await?;
            if user.is_none() {
                return Err(AppError::UserNotFound);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use sqlx_db_tester::TestMySql;

    #[tokio::test]
    async fn test_create_user() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );

        let pool = tdb.get_pool().await;

        let user_service = UserService::new(pool.clone());
        let user_id = user_service
            .create(
                1,
                "test123".to_string(),
                "123456".to_string(),
                "test123@example.com".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(
            User::find_by_id(user_id, &pool)
                .await
                .unwrap()
                .unwrap()
                .email,
            "test123@example.com"
        );
    }
    #[tokio::test]
    #[should_panic]
    async fn test_create_user_as_same_email() {
        let tdb = TestMySql::new(
            "mysql://root:123456@localhost:3306".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let user_service = UserService::new(pool);
        user_service
            .create(
                1,
                "test".to_string(),
                "123456".to_string(),
                "test@test".to_string(),
            )
            .await
            .unwrap();
    }
}
