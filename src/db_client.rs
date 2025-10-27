use thiserror::Error;
use sqlx::Row;

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
    pub async fn new(database_url: &str) -> Result<Self, DbClientError> {
        let pool = sqlx::SqlitePool::connect(database_url).await?;
        let client = DbClient { pool };
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
                FOREIGN KEY (creator_info_id) REFERENCES creator_info(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
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
}