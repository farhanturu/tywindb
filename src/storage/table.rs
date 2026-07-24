#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::Row;

/// Table storage for persistent data
pub struct TableStorage {
    data_dir: PathBuf,
    tables: HashMap<String, Vec<Row>>,
}

#[derive(Serialize, Deserialize)]
struct TableData {
    name: String,
    rows: Vec<Row>,
}

impl TableStorage {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;

        let mut storage = Self {
            data_dir,
            tables: HashMap::new(),
        };

        // Load existing tables
        storage.load_tables()?;

        Ok(storage)
    }

    fn load_tables(&mut self) -> Result<()> {
        let tables_dir = self.data_dir.join("tables");
        if !tables_dir.exists() {
            return Ok(());
        }

        // Read all .tbl files
        for entry in std::fs::read_dir(&tables_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("tbl") {
                let file = File::open(&path)?;
                let mut reader = BufReader::new(file);
                
                let mut data = Vec::new();
                reader.read_to_end(&mut data)?;
                
                if let Ok(table_data) = bincode::deserialize::<TableData>(&data) {
                    self.tables.insert(table_data.name, table_data.rows);
                }
            }
        }

        Ok(())
    }

    pub fn get_tables(&self) -> &HashMap<String, Vec<Row>> {
        &self.tables
    }

    pub fn get_tables_mut(&mut self) -> &mut HashMap<String, Vec<Row>> {
        &mut self.tables
    }

    pub fn get_table(&self, name: &str) -> Option<&Vec<Row>> {
        self.tables.get(name)
    }

    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Vec<Row>> {
        self.tables.get_mut(name)
    }

    pub fn create_table(&mut self, name: String) {
        self.tables.entry(name).or_default();
    }

    pub fn insert_rows(&mut self, table: &str, rows: Vec<Row>) -> usize {
        let table_rows = self.tables.entry(table.to_string()).or_default();
        let count = rows.len();
        table_rows.extend(rows);
        count
    }

    pub fn flush(&self) -> Result<()> {
        let tables_dir = self.data_dir.join("tables");
        std::fs::create_dir_all(&tables_dir)?;

        for (name, rows) in &self.tables {
            let table_data = TableData {
                name: name.clone(),
                rows: rows.clone(),
            };

            let data = bincode::serialize(&table_data)?;
            
            let file_path = tables_dir.join(format!("{}.tbl", name));
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&file_path)?;
            
            let mut writer = BufWriter::new(file);
            writer.write_all(&data)?;
            writer.flush()?;
        }

        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        self.flush()
    }
}
