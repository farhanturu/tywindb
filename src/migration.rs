#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::{Result, TywindbError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up: String,
    pub down: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MigrationStatus {
    pub version: u32,
    pub name: String,
    pub applied: bool,
    pub applied_at: Option<String>,
}

pub struct MigrationManager {
    migrations_dir: PathBuf,
}

impl MigrationManager {
    pub fn new(db_path: &Path) -> Self {
        let migrations_dir = db_path.join("migrations");
        Self { migrations_dir }
    }

    pub fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.migrations_dir)?;
        Ok(())
    }

    pub fn create(&self, name: &str, up: &str, down: &str) -> Result<PathBuf> {
        let files = self.list_files()?;
        let version = files.len() as u32 + 1;
        let filename = format!("{:04}_{}.sql", version, name.replace(' ', "_"));
        let filepath = self.migrations_dir.join(&filename);

        let content = format!(
            "-- Migration: {}\n-- Version: {}\n\n-- UP\n{};\n\n-- DOWN\n{};\n",
            name, version, up, down
        );

        fs::write(&filepath, content)?;
        Ok(filepath)
    }

    pub fn list_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        if self.migrations_dir.exists() {
            for entry in fs::read_dir(&self.migrations_dir)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".sql") {
                    files.push(name);
                }
            }
        }
        files.sort();
        Ok(files)
    }

    pub fn get_pending(&self, applied: &[u32]) -> Result<Vec<Migration>> {
        let files = self.list_files()?;
        let mut pending = Vec::new();

        for file in files {
            let version: u32 = file.split('_').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if !applied.contains(&version) {
                let content = fs::read_to_string(self.migrations_dir.join(&file))?;
                let (up, down) = parse_migration(&content);
                pending.push(Migration {
                    version,
                    name: file.replace(".sql", ""),
                    up,
                    down,
                });
            }
        }

        Ok(pending)
    }

    pub fn get_all(&self) -> Result<Vec<Migration>> {
        let files = self.list_files()?;
        let mut migrations = Vec::new();

        for file in files {
            let version: u32 = file.split('_').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            let content = fs::read_to_string(self.migrations_dir.join(&file))?;
            let (up, down) = parse_migration(&content);
            migrations.push(Migration {
                version,
                name: file.replace(".sql", ""),
                up,
                down,
            });
        }

        Ok(migrations)
    }

    pub fn get_by_version(&self, version: u32) -> Result<Option<Migration>> {
        let files = self.list_files()?;
        for file in files {
            let v: u32 = file.split('_').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            if v == version {
                let content = fs::read_to_string(self.migrations_dir.join(&file))?;
                let (up, down) = parse_migration(&content);
                return Ok(Some(Migration {
                    version,
                    name: file.replace(".sql", ""),
                    up,
                    down,
                }));
            }
        }
        Ok(None)
    }

    pub fn get_status(&self, applied: &[u32]) -> Result<Vec<MigrationStatus>> {
        let files = self.list_files()?;
        let mut statuses = Vec::new();

        for file in files {
            let version: u32 = file.split('_').next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);

            statuses.push(MigrationStatus {
                version,
                name: file.replace(".sql", ""),
                applied: applied.contains(&version),
                applied_at: None,
            });
        }

        Ok(statuses)
    }

    pub fn dry_run(&self, migration: &Migration) -> String {
        format!(
            "DRY RUN - Migration {} (v{}):\n\nUP:\n{}\n\nDOWN:\n{}",
            migration.name, migration.version, migration.up, migration.down
        )
    }

    pub fn get_version_from_file(filename: &str) -> Option<u32> {
        filename.split('_').next()?.parse().ok()
    }
}

fn parse_migration(content: &str) -> (String, String) {
    let mut up = String::new();
    let mut down = String::new();
    let mut section = "";

    for line in content.lines() {
        if line.contains("-- UP") {
            section = "up";
        } else if line.contains("-- DOWN") {
            section = "down";
        } else if section == "up" && !line.starts_with("--") {
            up.push_str(line);
            up.push('\n');
        } else if section == "down" && !line.starts_with("--") {
            down.push_str(line);
            down.push('\n');
        }
    }

    (up.trim().to_string(), down.trim().to_string())
}
