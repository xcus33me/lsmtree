use std::sync::atomic::AtomicU64;

use crossbeam_skiplist::SkipMap;

use crate::key::{InternalKey, InternalValue, SeqNo, UserKey, UserValue, ValueType};

pub(crate) struct MemTable {
    data: SkipMap<InternalKey, UserValue>,

    /// Approximately size of MemTable
    size_bytes: AtomicU64,
}

impl MemTable {
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: SkipMap::default(),
            size_bytes: AtomicU64::default(),
        }
    }

    // Returns the item by key (with the height seqno)
    pub fn get(&self, key: &UserKey, seqno: SeqNo) -> Option<InternalValue> {
        if seqno == 0 {
            return None;
        }

        let lower_bound = InternalKey {
            user_key: key.clone(),
            seqno: seqno - 1,
            value_type: ValueType::Value,
        };

        let entry = self.data.range(lower_bound..).next()?;

        if &entry.key().user_key != key {
            return None;
        }

        if entry.key().value_type.is_tombstone() {
            return None;
        }

        Some(InternalValue::new(
            entry.key().clone(),
            entry.value().clone(),
        ))
    }
}
