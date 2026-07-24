#![allow(dead_code)]

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use chrono::{DateTime, Utc};
use crate::error::{Result, TywindbError};

pub struct BackupManager {
    data_dir: PathBuf,
}

#[derive(Debug)]
pub struct BackupInfo {
    pub filename: String,
    pub size: u64,
    pub timestamp: DateTime<Utc>,
    pub compressed: bool,
}

impl BackupManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
        }
    }

    pub fn backup(&self, output: &Path, compress: bool) -> Result<BackupInfo> {
        let data_file = self.data_dir.join("tywindb.data");
        let auth_file = self.data_dir.join(".tywindb_auth");
        let tables_dir = self.data_dir.join("tables");

        let mut backup_data = Vec::new();

        backup_data.extend_from_slice(b"TYWINDB_BACKUP_V1\n");

        let has_auth = auth_file.exists();
        backup_data.extend_from_slice(if has_auth { b"AUTH:1\n" } else { b"AUTH:0\n" });

        if has_auth {
            let auth_content = fs::read(&auth_file)?;
            backup_data.extend_from_slice(&(auth_content.len() as u32).to_le_bytes());
            backup_data.extend_from_slice(&auth_content);
        }

        if data_file.exists() {
            let data = fs::read(&data_file)?;
            backup_data.extend_from_slice(&(data.len() as u64).to_le_bytes());
            backup_data.extend_from_slice(&data);
        } else {
            backup_data.extend_from_slice(&0u64.to_le_bytes());
        }

        let mut table_count: u32 = 0;
        if tables_dir.exists() {
            for entry in fs::read_dir(&tables_dir)? {
                let entry = entry?;
                if entry.path().extension().map(|e| e == "tbl").unwrap_or(false) {
                    table_count += 1;
                }
            }
        }
        backup_data.extend_from_slice(&table_count.to_le_bytes());

        if tables_dir.exists() {
            for entry in fs::read_dir(&tables_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "tbl").unwrap_or(false) {
                    let name = path.file_stem().unwrap().to_string_lossy();
                    let data = fs::read(&path)?;

                    backup_data.extend_from_slice(&(name.len() as u32).to_le_bytes());
                    backup_data.extend_from_slice(name.as_bytes());
                    backup_data.extend_from_slice(&(data.len() as u64).to_le_bytes());
                    backup_data.extend_from_slice(&data);
                }
            }
        }

        if compress {
            let file = fs::File::create(output)?;
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(&backup_data)?;
            encoder.finish()?;
        } else {
            fs::write(output, &backup_data)?;
        }

        let metadata = fs::metadata(output)?;

        Ok(BackupInfo {
            filename: output.file_name().unwrap().to_string_lossy().to_string(),
            size: metadata.len(),
            timestamp: Utc::now(),
            compressed: compress,
        })
    }

    pub fn restore(&self, backup_path: &Path, password: Option<&str>) -> Result<()> {
        let raw_data = fs::read(backup_path)?;

        let backup_data = if raw_data.len() >= 2 && raw_data[0] == 0x1f && raw_data[1] == 0x8b {
            let mut decoder = GzDecoder::new(&raw_data[..]);
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;
            data
        } else {
            raw_data
        };

        let header = b"TYWINDB_BACKUP_V1\n";
        if !backup_data.starts_with(header) {
            return Err(TywindbError::Backup("Invalid backup format".to_string()));
        }

        let mut pos = header.len();

        let auth_header = &backup_data[pos..pos+7];
        let has_auth = auth_header.starts_with(b"AUTH:1");
        pos += 7;

        let auth_file = self.data_dir.join(".tywindb_auth");
        if has_auth {
            let auth_len = u32::from_le_bytes(backup_data[pos..pos+4].try_into().unwrap()) as usize;
            pos += 4;
            let auth_data = &backup_data[pos..pos+auth_len];
            pos += auth_len;

            if let Some(pwd) = password {
                let stored_hash = String::from_utf8_lossy(auth_data);
                if crate::crypto::Crypto::verify_password(pwd, &stored_hash)? {
                    fs::write(&auth_file, auth_data)?;
                } else {
                    return Err(TywindbError::AuthFailed("Password mismatch".to_string()));
                }
            } else {
                return Err(TywindbError::AuthFailed("Password required".to_string()));
            }
        }

        let data_len = u64::from_le_bytes(backup_data[pos..pos+8].try_into().unwrap()) as usize;
        pos += 8;

        if data_len > 0 {
            let data = &backup_data[pos..pos+data_len];
            pos += data_len;
            fs::write(self.data_dir.join("tywindb.data"), data)?;
        }

        let table_count = u32::from_le_bytes(backup_data[pos..pos+4].try_into().unwrap()) as usize;
        pos += 4;

        let tables_dir = self.data_dir.join("tables");
        fs::create_dir_all(&tables_dir)?;

        for _ in 0..table_count {
            let name_len = u32::from_le_bytes(backup_data[pos..pos+4].try_into().unwrap()) as usize;
            pos += 4;
            let name = String::from_utf8(backup_data[pos..pos+name_len].to_vec()).unwrap();
            pos += name_len;

            let tbl_data_len = u64::from_le_bytes(backup_data[pos..pos+8].try_into().unwrap()) as usize;
            pos += 8;
            let tbl_data = &backup_data[pos..pos+tbl_data_len];
            pos += tbl_data_len;

            fs::write(tables_dir.join(format!("{}.tbl", name)), tbl_data)?;
        }

        Ok(())
    }

    pub fn list_backups(&self, backup_dir: &Path) -> Result<Vec<BackupInfo>> {
        let mut backups = Vec::new();

        if backup_dir.exists() {
            for entry in fs::read_dir(backup_dir)? {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();

                if name.starts_with("tywindb_backup_") && (name.ends_with(".tdb") || name.ends_with(".tdb.gz")) {
                    let metadata = fs::metadata(&path)?;
                    let compressed = name.ends_with(".gz");

                    backups.push(BackupInfo {
                        filename: name,
                        size: metadata.len(),
                        timestamp: metadata.modified().unwrap().into(),
                        compressed,
                    });
                }
            }
        }

        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(backups)
    }

    pub fn create_backup_path(&self, backup_dir: &Path) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        backup_dir.join(format!("tywindb_backup_{}.tdb.gz", timestamp))
    }
}
