use crate::models::*;
use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::{MySqlPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: MySqlPool,
}

impl Database {
    pub async fn new(database_url: &str, _max_connections: u32) -> Result<Self> {
        let pool = MySqlPool::connect(database_url)
            .await
            .context("Failed to connect to database")?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("Failed to run migrations")?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    // User operations
    pub async fn create_user(&self, user: CreateUser) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let password_hash = hash_password(&user.password)?;

        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, is_admin)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&user.username)
        .bind(&password_hash)
        .bind(user.is_admin)
        .execute(&self.pool)
        .await
        .context("Failed to create user")?;

        self.get_user_by_id(&id).await
    }

    pub async fn get_user_by_id(&self, id: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, is_admin, created_at, updated_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("User not found")?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, is_admin, created_at, updated_at
            FROM users
            WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .context("User not found")?;

        Ok(user)
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, is_admin, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list users")?;

        Ok(users)
    }

    pub async fn delete_user(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete user")?;

        Ok(())
    }

    pub async fn verify_password(&self, username: &str, password: &str) -> Result<User> {
        let user = self.get_user_by_username(username).await?;

        verify_password(password, &user.password_hash)
            .context("Invalid password")?;

        Ok(user)
    }

    // Upload log operations
    pub async fn create_upload_log(&self, log: CreateUploadLog) -> Result<i32> {
        let result = sqlx::query(
            r#"
            INSERT INTO upload_logs (user_id, upload_type, source)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&log.user_id)
        .bind(&log.upload_type)
        .bind(&log.source)
        .execute(&self.pool)
        .await
        .context("Failed to create upload log")?;

        Ok(result.last_insert_id() as i32)
    }

    pub async fn update_upload_log_status(
        &self,
        id: i32,
        status: &str,
        file_count: Option<i32>,
        error_message: Option<String>,
    ) -> Result<()> {
        let mut query = String::from("UPDATE upload_logs SET status = ?");
        let mut bindings = vec![status.to_string()];

        if let Some(count) = file_count {
            query.push_str(", file_count = ?");
            bindings.push(count.to_string());
        }

        if let Some(error) = error_message {
            query.push_str(", error_message = ?");
            bindings.push(error);
        }

        if status == "completed" || status == "failed" {
            query.push_str(", completed_at = NOW()");
        }

        query.push_str(" WHERE id = ?");
        bindings.push(id.to_string());

        let mut q = sqlx::query(&query);
        for binding in bindings {
            q = q.bind(binding);
        }

        q.execute(&self.pool)
            .await
            .context("Failed to update upload log")?;

        Ok(())
    }

    pub async fn get_upload_logs(&self, user_id: Option<&str>, limit: i64) -> Result<Vec<UploadLog>> {
        let logs = if let Some(uid) = user_id {
            sqlx::query_as::<_, UploadLog>(
                r#"
                SELECT id, user_id, upload_type, source, status, file_count, error_message, created_at, completed_at
                FROM upload_logs
                WHERE user_id = ?
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(uid)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, UploadLog>(
                r#"
                SELECT id, user_id, upload_type, source, status, file_count, error_message, created_at, completed_at
                FROM upload_logs
                ORDER BY created_at DESC
                LIMIT ?
                "#,
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(logs)
    }

    // Config operations
    pub async fn get_config(&self, key: &str) -> Result<Option<String>> {
        let result = sqlx::query(
            r#"
            SELECT value FROM config WHERE `key` = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to get config")?;

        Ok(result.map(|row| row.get("value")))
    }

    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO config (`key`, value)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE value = VALUES(value)
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await
        .context("Failed to set config")?;

        Ok(())
    }

    pub async fn list_config(&self) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT `key`, value FROM config ORDER BY `key`
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list config")?;

        let configs = rows
            .into_iter()
            .map(|row| (row.get("key"), row.get("value")))
            .collect();

        Ok(configs)
    }
}

// Password hashing utilities
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<()> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))?;
    Ok(())
}
