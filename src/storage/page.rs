#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Page size in bytes (4KB default)
pub const PAGE_SIZE: usize = 4096;

/// Page number
pub type PageNum = u64;

/// Page type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PageType {
    Meta,
    Table,
    Index,
    Free,
}

/// Database page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub num: PageNum,
    pub page_type: PageType,
    pub data: Vec<u8>,
    pub dirty: bool,
}

impl Page {
    pub fn new(num: PageNum, page_type: PageType) -> Self {
        Self {
            num,
            page_type,
            data: vec![0u8; PAGE_SIZE],
            dirty: false,
        }
    }

    pub fn set_data(&mut self, offset: usize, data: &[u8]) {
        let end = std::cmp::min(offset + data.len(), self.data.len());
        self.data[offset..end].copy_from_slice(&data[..end - offset]);
        self.dirty = true;
    }

    pub fn get_data(&self, offset: usize, len: usize) -> &[u8] {
        let end = std::cmp::min(offset + len, self.data.len());
        &self.data[offset..end]
    }
}
