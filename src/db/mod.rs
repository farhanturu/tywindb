#![allow(dead_code)]

use std::path::Path;

use crate::crypto::Crypto;
use crate::engine::{Executor, QueryResult};
use crate::error::{Result, TywindbError};
use crate::sql::parser::SqlParser;
use crate::storage::engine::StorageEngine;
use crate::storage::table::TableStorage;

pub struct Database {
    executor: Executor,
    table_storage: TableStorage,
    data_dir: std::path::PathBuf,
    crypto: Option<Crypto>,
    password_hash: Option<String>,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let data_dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let password_file = data_dir.join(".tywindb_auth");
        let has_password = password_file.exists();

        let storage = StorageEngine::open(&data_dir)?;
        let table_storage = TableStorage::open(&data_dir)?;

        let mut executor = Executor::new(storage);
        for (name, rows) in table_storage.get_tables() {
            for row in rows {
                executor.tables_mut().entry(name.clone()).or_default().push(row.clone());
            }
        }

        Ok(Self {
            executor,
            table_storage,
            data_dir,
            crypto: None,
            password_hash: if has_password {
                std::fs::read_to_string(&password_file).ok()
            } else {
                None
            },
        })
    }

    pub fn set_password(&mut self, password: &str) -> Result<()> {
        let hash = Crypto::hash_password(password)?;
        self.password_hash = Some(hash.clone());
        self.crypto = Some(Crypto::from_password(password));

        let password_file = self.data_dir.join(".tywindb_auth");
        std::fs::write(&password_file, hash)?;

        Ok(())
    }

    pub fn authenticate(&mut self, password: &str) -> Result<bool> {
        if let Some(ref hash) = self.password_hash {
            let valid = Crypto::verify_password(password, hash)?;
            if valid {
                self.crypto = Some(Crypto::from_password(password));
            }
            Ok(valid)
        } else {
            Ok(true)
        }
    }

    pub fn is_locked(&self) -> bool {
        self.password_hash.is_some() && self.crypto.is_none()
    }

    pub fn has_password(&self) -> bool {
        self.password_hash.is_some()
    }

    pub fn query(&mut self, sql: &str) -> Result<QueryResult> {
        if self.is_locked() {
            return Err(TywindbError::AuthFailed("Database is locked. Authenticate first.".to_string()));
        }

        let mut parser = SqlParser::new(sql)?;
        let statement = parser.parse()?;
        let result = self.executor.execute(statement)?;
        self.sync_to_storage()?;
        Ok(result)
    }

    pub fn execute_batch(&mut self, sql: &str) -> Result<Vec<QueryResult>> {
        let mut results = Vec::new();
        for statement in sql.split(';') {
            let statement = statement.trim();
            if !statement.is_empty() {
                let result = self.query(statement)?;
                results.push(result);
            }
        }
        Ok(results)
    }

    pub fn begin(&mut self) -> Result<u64> {
        match self.query("BEGIN")? {
            QueryResult::TransactionStarted { tx_id } => Ok(tx_id),
            _ => unreachable!(),
        }
    }

    pub fn commit(&mut self) -> Result<()> {
        match self.query("COMMIT")? {
            QueryResult::TransactionCommitted => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn rollback(&mut self) -> Result<()> {
        match self.query("ROLLBACK")? {
            QueryResult::TransactionRolledBack => Ok(()),
            _ => unreachable!(),
        }
    }

    fn sync_to_storage(&mut self) -> Result<()> {
        for (name, rows) in self.executor.tables() {
            let storage_rows = self.table_storage.get_tables_mut()
                .entry(name.clone())
                .or_default();
            *storage_rows = rows.clone();
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.sync_to_storage()?;
        self.table_storage.flush()?;
        Ok(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
