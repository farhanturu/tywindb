#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::error::Result;

/// WAL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub lsn: u64,           // Log Sequence Number
    pub tx_id: u64,         // Transaction ID
    pub operation: WalOp,
}

/// WAL operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Begin,
    Commit,
    Abort,
    Insert { table: String, row_id: u64, data: Vec<u8> },
    Update { table: String, row_id: u64, data: Vec<u8> },
    Delete { table: String, row_id: u64 },
}

/// Write-Ahead Log
pub struct Wal {
    writer: BufWriter<File>,
    reader: BufReader<File>,
    path: String,
    next_lsn: u64,
}

impl Wal {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_string_lossy().to_string();
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        
        let writer = BufWriter::new(file);

        let reader_file = OpenOptions::new()
            .read(true)
            .open(&path)?;
        let reader = BufReader::new(reader_file);

        // TODO: Read existing entries to find max LSN
        let next_lsn = 1;

        Ok(Self {
            writer,
            reader,
            path,
            next_lsn,
        })
    }

    pub fn append(&mut self, tx_id: u64, operation: WalOp) -> Result<u64> {
        let lsn = self.next_lsn;
        self.next_lsn += 1;

        let entry = WalEntry {
            lsn,
            tx_id,
            operation,
        };

        let encoded = bincode::serialize(&entry)?;
        let len = encoded.len() as u64;

        self.writer.write_all(&len.to_le_bytes())?;
        self.writer.write_all(&encoded)?;
        self.writer.flush()?;

        Ok(lsn)
    }

    pub fn sync(&mut self) -> Result<()> {
        self.writer.flush()?;
        Ok(())
    }

    pub fn read_all(&mut self) -> Result<Vec<WalEntry>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            let mut len_buf = [0u8; 8];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let len = u64::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            reader.read_exact(&mut data)?;

            let entry: WalEntry = bincode::deserialize(&data)?;
            entries.push(entry);
        }

        Ok(entries)
    }
}
