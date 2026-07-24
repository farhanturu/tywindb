#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use crate::error::{Result, TywindbError};
use crate::types::{Row, Value};

/// Version of a row
#[derive(Debug, Clone)]
pub struct RowVersion {
    pub data: Row,
    pub tx_id: u64,
    pub version: u64,
    pub deleted: bool,
}

/// MVCC Table
#[derive(Debug, Clone)]
pub struct MvccTable {
    pub name: String,
    versions: Vec<RowVersion>,
}

impl MvccTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            versions: Vec::new(),
        }
    }

    pub fn insert(&mut self, row: Row, tx_id: u64, version: u64) {
        self.versions.push(RowVersion {
            data: row,
            tx_id,
            version,
            deleted: false,
        });
    }

    pub fn delete(&mut self, row_id: &str, tx_id: u64, version: u64) {
        self.versions.push(RowVersion {
            data: {
                let mut row = Row::new();
                row.insert("_row_id".to_string(), Value::Text(row_id.to_string()));
                row
            },
            tx_id,
            version,
            deleted: true,
        });
    }

    pub fn get_visible_rows(&self, _tx_id: u64) -> Vec<&Row> {
        // Simple MVCC: return rows that are visible to this transaction
        // A row is visible if:
        // 1. It was created by a committed transaction before tx_id
        // 2. It hasn't been deleted by a committed transaction before tx_id
        
        let mut visible = HashMap::new();
        
        for version in &self.versions {
            let row_id = version.data.get("_row_id")
                .and_then(|v| v.as_str())
                .unwrap_or("_default");
            
            // For simplicity, we'll use the latest version for each row
            // In a real MVCC implementation, we'd track transaction visibility
            if !version.deleted {
                visible.insert(row_id.to_string(), &version.data);
            }
        }
        
        visible.into_values().collect()
    }
}

/// MVCC Transaction
#[derive(Debug)]
pub struct MvccTransaction {
    pub id: u64,
    pub read_version: u64,
    pub writes: Vec<MvccWrite>,
}

#[derive(Debug, Clone)]
pub enum MvccWrite {
    Insert { table: String, row: Row },
    Delete { table: String, row_id: String },
    Update { table: String, row_id: String, row: Row },
}

/// MVCC Manager
pub struct MvccManager {
    next_tx_id: AtomicU64,
    next_version: AtomicU64,
    tables: RwLock<HashMap<String, MvccTable>>,
}

impl Default for MvccManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MvccManager {
    pub fn new() -> Self {
        Self {
            next_tx_id: AtomicU64::new(1),
            next_version: AtomicU64::new(1),
            tables: RwLock::new(HashMap::new()),
        }
    }

    pub fn begin_transaction(&self) -> MvccTransaction {
        let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst);
        let read_version = self.next_version.load(Ordering::SeqCst);
        
        MvccTransaction {
            id: tx_id,
            read_version,
            writes: Vec::new(),
        }
    }

    pub fn next_version(&self) -> u64 {
        self.next_version.fetch_add(1, Ordering::SeqCst)
    }

    pub fn create_table(&self, name: &str) {
        let mut tables = self.tables.write().unwrap();
        tables.entry(name.to_string()).or_insert_with(|| MvccTable::new(name.to_string()));
    }

    pub fn insert(&self, tx_id: u64, table: &str, row: Row) -> Result<()> {
        let version = self.next_version();
        let mut tables = self.tables.write().unwrap();
        
        let table = tables.get_mut(table)
            .ok_or_else(|| TywindbError::TableNotFound(table.to_string()))?;
        
        table.insert(row, tx_id, version);
        Ok(())
    }

    pub fn delete(&self, tx_id: u64, table: &str, row_id: &str) -> Result<()> {
        let version = self.next_version();
        let mut tables = self.tables.write().unwrap();
        
        let table = tables.get_mut(table)
            .ok_or_else(|| TywindbError::TableNotFound(table.to_string()))?;
        
        table.delete(row_id, tx_id, version);
        Ok(())
    }

    pub fn read(&self, table: &str, tx_id: u64) -> Result<Vec<Row>> {
        let tables = self.tables.read().unwrap();
        
        let table = tables.get(table)
            .ok_or_else(|| TywindbError::TableNotFound(table.to_string()))?;
        
        Ok(table.get_visible_rows(tx_id).into_iter().cloned().collect())
    }

    pub fn commit(&self, transaction: MvccTransaction) -> Result<()> {
        // Apply all writes
        for write in transaction.writes {
            match write {
                MvccWrite::Insert { table, row } => {
                    self.insert(transaction.id, &table, row)?;
                }
                MvccWrite::Delete { table, row_id } => {
                    self.delete(transaction.id, &table, &row_id)?;
                }
                MvccWrite::Update { table, row_id, row } => {
                    // Delete old version and insert new
                    self.delete(transaction.id, &table, &row_id)?;
                    self.insert(transaction.id, &table, row)?;
                }
            }
        }
        
        Ok(())
    }
}
