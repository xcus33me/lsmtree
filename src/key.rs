use bytes::Bytes;

// User defined key type
pub type UserKey = Bytes;

// User defined value type
pub type UserValue = Bytes;

pub type SeqNo = u64;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ValueType {
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
    pub seqno: SeqNo,
    pub value_type: ValueType,
}

impl InternalKey {
    #[must_use]
    pub fn new<K: Into<UserKey>>(user_key: K, seqno: SeqNo, value_type: ValueType) -> Self {
        Self {
            user_key: user_key.into(),
            seqno: seqno,
            value_type: value_type,
        }
    }

    pub fn is_tombstone(&self) -> bool {
        self.value_type.is_tombstone()
    }
}

impl PartialOrd for InternalKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InternalKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.user_key
            .cmp(&other.user_key)
            .then(self.seqno.cmp(&other.seqno).reverse())
    }
}

pub struct InternalValue {
    pub key: InternalKey,
    pub value: UserValue,
}

impl InternalValue {
    pub fn new<V: Into<UserValue>>(key: InternalKey, value: V) -> Self {
        Self {
            key: key,
            value: value.into(),
        }
    }
}
