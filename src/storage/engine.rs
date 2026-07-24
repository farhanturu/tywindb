#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::{Result, TywindbError};
use crate::storage::page::{Page, PageNum, PageType, PAGE_SIZE};
use crate::storage::wal::{Wal, WalOp};

/// Storage engine
pub struct StorageEngine {
    data_dir: PathBuf,
    wal: Wal,
    pages: HashMap<PageNum, Page>,
    next_page: PageNum,
}

impl StorageEngine {
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;

        // Open WAL
        let wal_path = data_dir.join("tywindb.wal");
        let wal = Wal::open(&wal_path)?;

        let mut engine = Self {
            data_dir,
            wal,
            pages: HashMap::new(),
            next_page: 1,
        };

        // Load existing pages
        engine.load_pages()?;

        Ok(engine)
    }

    fn load_pages(&mut self) -> Result<()> {
        let data_file = self.data_dir.join("tywindb.data");
        if !data_file.exists() {
            return Ok(());
        }

        let file = File::open(&data_file)?;
        let mut reader = BufReader::new(file);
        let mut page_num = 0u64;

        loop {
            let mut page_data = vec![0u8; PAGE_SIZE];
            match reader.read_exact(&mut page_data) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            // Parse page type from first byte
            let page_type = match page_data[0] {
                0 => PageType::Meta,
                1 => PageType::Table,
                2 => PageType::Index,
                _ => PageType::Free,
            };

            let page = Page {
                num: page_num,
                page_type,
                data: page_data,
                dirty: false,
            };

            self.pages.insert(page_num, page);
            page_num += 1;
        }

        self.next_page = page_num;
        Ok(())
    }

    pub fn alloc_page(&mut self, page_type: PageType) -> Result<PageNum> {
        let page_num = self.next_page;
        self.next_page += 1;

        let page = Page::new(page_num, page_type);
        self.pages.insert(page_num, page);

        Ok(page_num)
    }

    pub fn get_page(&self, page_num: PageNum) -> Result<&Page> {
        self.pages
            .get(&page_num)
            .ok_or_else(|| TywindbError::Corrupted(format!("Page {} not found", page_num)))
    }

    pub fn get_page_mut(&mut self, page_num: PageNum) -> Result<&mut Page> {
        self.pages
            .get_mut(&page_num)
            .ok_or_else(|| TywindbError::Corrupted(format!("Page {} not found", page_num)))
    }

    pub fn write_wal(&mut self, tx_id: u64, operation: WalOp) -> Result<u64> {
        self.wal.append(tx_id, operation)
    }

    pub fn sync_wal(&mut self) -> Result<()> {
        self.wal.sync()
    }

    pub fn flush(&mut self) -> Result<()> {
        // Write all dirty pages to data file
        let data_file = self.data_dir.join("tywindb.data");
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&data_file)?;
        let mut writer = BufWriter::new(file);

        // Sort pages by number
        let mut sorted_pages: Vec<_> = self.pages.iter().collect();
        sorted_pages.sort_by_key(|(num, _)| *num);

        for (_, page) in sorted_pages {
            writer.seek(SeekFrom::Start(page.num * PAGE_SIZE as u64))?;
            writer.write_all(&page.data)?;
        }

        writer.flush()?;

        // Mark all pages as clean
        for page in self.pages.values_mut() {
            page.dirty = false;
        }

        // Sync WAL
        self.sync_wal()?;

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.flush()?;
        Ok(())
    }
}
