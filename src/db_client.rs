use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbClientError {
    
}

#[derive(Debug)]
pub struct DbClient {
    pub pool: sqlx::SqlitePool,
}