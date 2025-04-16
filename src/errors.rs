use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Other error: {0}")]
    OtherError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Page Not Found: {0}")]
    PageNotFound(String),
}
