use std::sync::atomic::AtomicU64;

use crossbeam_skiplist::SkipMap;

use crate::key::{InternalKey, UserValue};

pub(crate) struct MemTable {
    data: SkipMap<InternalKey, UserValue>,

    /// Approximately size of MemTable
    size_bytes: AtomicU64,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            data: SkipMap::default(),
            size_bytes: AtomicU64::default(),
        }
    }
}
