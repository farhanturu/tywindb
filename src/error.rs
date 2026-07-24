#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TywindbError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQL parse error: {0}")]
    SqlParse(String),

    #[error("Table not found: {0}")]
    TableNotFound(String),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Database corrupted: {0}")]
    Corrupted(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Migration error: {0}")]
    Migration(String),
}

impl From<String> for TywindbError {
    fn from(s: String) -> Self {
        TywindbError::SqlParse(s)
    }
}

pub type Result<T> = std::result::Result<T, TywindbError>;
