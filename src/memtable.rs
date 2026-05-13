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
        let probe = InternalKey::new(key.clone(), seqno, ValueType::Value);

        let entry = self.data.range(probe..).next()?;

        if entry.key().user_key != *key {
            return None;
        }

        let internal_key = entry.key().clone();

        if internal_key.is_tombstone() {
            return None;
        }

        Some(InternalValue::new(internal_key, entry.value().clone()))
    }

    pub fn insert(&self, item: InternalValue) {
        let item_size = item.key.user_key.len()
            + std::mem::size_of::<SeqNo>()
            + std::mem::size_of::<ValueType>()
            + item.value.len();

        self.size_bytes
            .fetch_add(item_size as u64, std::sync::atomic::Ordering::Relaxed);

        self.data.insert(item.key, item.value);
    }

    pub fn size_bytes(&self) -> u64 {
        return self.size_bytes.load(std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    // for the "flush" in SSTable
    pub fn iter(&self) -> impl Iterator<Item = InternalValue> + '_ {
        self.data
            .iter()
            .map(|entry| InternalValue::new(entry.key().clone(), entry.value().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::{InternalKey, InternalValue, ValueType};
    use bytes::Bytes;

    fn make_value(key: &str, seqno: u64, val: &str) -> InternalValue {
        InternalValue::new(
            InternalKey::new(Bytes::from(key.to_owned()), seqno, ValueType::Value),
            Bytes::from(val.to_owned()),
        )
    }

    fn make_tombstone(key: &str, seqno: u64) -> InternalValue {
        InternalValue::new(
            InternalKey::new(Bytes::from(key.to_owned()), seqno, ValueType::Tombstone),
            Bytes::new(),
        )
    }

    #[test]
    fn test_insert_and_get() {
        let mt = MemTable::new();
        mt.insert(make_value("foo", 1, "bar"));
        let result = mt.get(&Bytes::from("foo"), 1);
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, Bytes::from("bar"));
    }

    fn test_snapshot_isolation() {
        let mt = MemTable::new();
        mt.insert(make_value("foo", 1, "old"));
        mt.insert(make_value("foo", 5, "new"));

        // snapshot до второй записи
        let old = mt.get(&Bytes::from("foo"), 3).unwrap();
        assert_eq!(old.value, Bytes::from("old"));

        // snapshot после
        let new = mt.get(&Bytes::from("foo"), 5).unwrap();
        assert_eq!(new.value, Bytes::from("new"));
    }

    #[test]
    fn test_tombstone_returns_none() {
        let mt = MemTable::new();
        mt.insert(make_value("foo", 1, "bar"));
        mt.insert(make_tombstone("foo", 2));
        assert!(mt.get(&Bytes::from("foo"), 10).is_none());
    }

    #[test]
    fn test_missing_key() {
        let mt = MemTable::new();
        assert!(mt.get(&Bytes::from("ghost"), 100).is_none());
    }

    #[test]
    fn test_size_grows() {
        let mt = MemTable::new();
        assert_eq!(mt.size_bytes(), 0);
        mt.insert(make_value("foo", 1, "bar"));
        assert!(mt.size_bytes() > 0);
    }
}
