//! Tywindb — A modern, fast, and easy-to-use database
//!
//! Tywindb combines the best features from SQLite, DuckDB, SurrealDB, and Redis
//! into one easy-to-use package.
//!
//! # Features
//!
//! - **SQL Parser** — Full SQL support with modern extensions
//! - **ACID Transactions** — BEGIN, COMMIT, ROLLBACK
//! - **Persistent Storage** — Data survives restarts
//! - **MVCC** — Multi-Version Concurrency Control for concurrent writes
//! - **Document Operations** — JSON/JSONB support
//! - **Vector Search** — HNSW index for similarity search
//! - **Full-Text Search** — BM25 ranking algorithm
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use tywindb::db::Database;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open or create database
//!     let mut db = Database::open("mydb.tdb")?;
//!
//!     // Create table
//!     db.query("CREATE TABLE users (id INTEGER, name TEXT, email TEXT)")?;
//!
//!     // Insert data
//!     db.query("INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')")?;
//!
//!     // Query data
//!     let result = db.query("SELECT * FROM users")?;
//!     println!("{:?}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod db;
pub mod engine;
pub mod error;
pub mod sql;
pub mod storage;
pub mod transaction;
pub mod types;
pub mod mvcc;
pub mod document;
pub mod vector;
pub mod search;
pub mod server;
pub mod crypto;

// Re-export main types for convenience
pub use db::Database;
pub use error::{TywindbError, Result};
pub use types::{Value, Row};
pub use engine::QueryResult;
