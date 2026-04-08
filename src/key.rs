use bytes::Bytes;

// User defined key type
pub type UserKey = Bytes;

// User defined value type
pub type UserValue = Bytes;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ValueType {
    /// Value exists
    Value,

    /// Value deleted
    Tombstone,
    // [TODO] WeakTombstone is a soft delete ('SingleDelete' from RocksDB).
    // Removes exactly one previous version
    // WeakTombstone,
}

impl ValueType {
    pub fn is_tombstone(self) -> bool {
        self == Self::Tombstone
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct InternalKey {
    pub user_key: UserKey,
    pub seqno: u64,
    pub value_type: ValueType,
}
