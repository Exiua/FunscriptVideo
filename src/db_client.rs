use std::path::Path;

use thiserror::Error;
use sqlx::{sqlite::SqliteConnectOptions, Row};

use crate::metadata::CreatorInfo;

#[derive(Debug, Error)]
pub enum DbClientError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug)]
pub struct DbClient {
    pub pool: sqlx::SqlitePool,
}

impl DbClient {
    pub async fn new<P: AsRef<Path>>(database_path: P) -> Result<Self, DbClientError> {
        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);
        let pool = sqlx::SqlitePool::connect_with(options).await?;
        let client: DbClient = Self { pool };
        client.create_tables().await?;

        Ok(client)
    }

    async fn create_tables(&self) -> Result<(), DbClientError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS creator_info (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                key TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS creator_info_socials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                creator_info_id INTEGER NOT NULL,
                social_url TEXT NOT NULL,
                FOREIGN KEY (creator_info_id) REFERENCES creator_info(id) ON DELETE CASCADE,
                UNIQUE (creator_info_id, social_url)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_creator_id_by_key(&self, key: &str) -> Result<Option<i64>, DbClientError> {
        let row = sqlx::query(
            r#"
            SELECT id FROM creator_info WHERE key = ?
            "#
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let creator_id = r.get::<i64, _>("id");
            Ok(Some(creator_id))
        }
        else {
            Ok(None)
        }
    }

    async fn get_creator_id_by_name(&self, name: &str) -> Result<Option<i64>, DbClientError> {
        let row = sqlx::query(
            r#"
            SELECT id FROM creator_info WHERE name = ?
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let creator_id = r.get::<i64, _>("id");
            Ok(Some(creator_id))
        }
        else {
            Ok(None)
        }
    }

    async fn get_creator_id(&self, key_name: &str) -> Result<Option<i64>, DbClientError> {
        if let Some(creator_id) = self.get_creator_id_by_key(key_name).await? {
            return Ok(Some(creator_id));
        }

        if let Some(creator_id) = self.get_creator_id_by_name(key_name).await? {
            return Ok(Some(creator_id));
        }

        Ok(None)
    }

    pub async fn get_creator_info_by_key(&self, key: &str) -> Result<Option<CreatorInfo>, DbClientError> {
        let row = sqlx::query(
            r#"
            SELECT id, name FROM creator_info WHERE key = ?
            "#
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let creator_id = row.get::<i64, _>("id");
        let creator_name = row.get::<String, _>("name");

        let socials_rows = sqlx::query(
            r#"
            SELECT social_url FROM creator_info_socials WHERE creator_info_id = ?
            "#,
        )
        .bind(creator_id)
        .fetch_all(&self.pool)
        .await?;

        let socials = socials_rows.into_iter().map(|r| r.get::<String, _>("social_url")).collect();

        Ok(Some(CreatorInfo::new(creator_name, socials)))
    }

    pub async fn get_creator_info_by_name(&self, name: &str) -> Result<Option<CreatorInfo>, DbClientError> {
        let row = sqlx::query(
            r#"
            SELECT id, name FROM creator_info WHERE name = ?
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        let row = match row {
            Some(r) => r,
            None => return Ok(None),
        };

        let creator_id = row.get::<i64, _>("id");
        let creator_name = row.get::<String, _>("name");

        let socials_rows = sqlx::query(
            r#"
            SELECT social_url FROM creator_info_socials WHERE creator_info_id = ?
            "#,
        )
        .bind(creator_id)
        .fetch_all(&self.pool)
        .await?;

        let socials = socials_rows.into_iter().map(|r| r.get::<String, _>("social_url")).collect();

        Ok(Some(CreatorInfo::new(creator_name, socials)))
    }

    pub async fn get_creator_info(&self, key_name: &str) -> Result<Option<CreatorInfo>, DbClientError> {
        if let Some(creator_info) = self.get_creator_info_by_key(key_name).await? {
            return Ok(Some(creator_info));
        }

        if let Some(creator_info) = self.get_creator_info_by_name(key_name).await? {
            return Ok(Some(creator_info));
        }

        Ok(None)
    }

    pub async fn insert_creator_info(&self, key: &str, creator_info: &CreatorInfo) -> Result<(), DbClientError> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            r#"
            INSERT INTO creator_info (name, key) VALUES (?, ?)
            "#,
        )
        .bind(&creator_info.name)
        .bind(key)
        .execute(&mut *tx)
        .await?;

        let creator_id = result.last_insert_rowid();

        for social in &creator_info.socials {
            sqlx::query(
                r#"
                INSERT INTO creator_info_socials (creator_info_id, social_url) VALUES (?, ?)
                "#,
            )
            .bind(creator_id)
            .bind(social)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_creator_info_by_key(&self, key: &str) -> Result<bool, DbClientError> {
        let result = sqlx::query(
            r#"
            DELETE FROM creator_info WHERE key = ?
            "#,
        )
        .bind(key)
        .execute(&self.pool)
        .await?;



        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_creator_info_by_name(&self, name: &str) -> Result<bool, DbClientError> {
        let result = sqlx::query(
            r#"
            DELETE FROM creator_info WHERE name = ?
            "#,
        )
        .bind(name)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn delete_creator_info(&self, key_name: &str) -> Result<bool, DbClientError> {
        if self.delete_creator_info_by_key(key_name).await? {
            return Ok(true);
        }

        if self.delete_creator_info_by_name(key_name).await? {
            return Ok(true);
        }

        Ok(false)
    }

    pub async fn add_social_to_creator(&self, key_name: &str, social_url: &str) -> Result<bool, DbClientError> {
        if let Some(creator_id) = self.get_creator_id(key_name).await? {
            let result = sqlx::query(
                r#"
                INSERT OR IGNORE INTO creator_info_socials (creator_info_id, social_url) VALUES (?, ?)
                "#,
            )
            .bind(creator_id)
            .bind(social_url)
            .execute(&self.pool)
            .await?;

            return Ok(result.rows_affected() > 0);
        }

        Ok(false)
    }

    pub async fn remove_social_from_creator(&self, key_name: &str, social_url: &str) -> Result<bool, DbClientError> {
        if let Some(creator_id) = self.get_creator_id(key_name).await? {
            let result = sqlx::query(
                r#"
                DELETE FROM creator_info_socials WHERE creator_info_id = ? AND social_url = ?
                "#,
            )
            .bind(creator_id)
            .bind(social_url)
            .execute(&self.pool)
            .await?;

            return Ok(result.rows_affected() > 0);
        }

        Ok(false)
    }
}