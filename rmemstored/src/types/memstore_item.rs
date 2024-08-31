use bytes::Bytes;

use super::memstore_value::MemstoreValue;

#[derive(Clone, Debug)]
pub struct MemstoreItem {
    value: MemstoreValue,
}

impl MemstoreItem {
    pub fn new(value: MemstoreValue) -> Self {
        Self { value }
    }

    pub fn into_value(self) -> MemstoreValue {
        self.value
    }

    pub fn weigher(key: &Bytes, item: &MemstoreItem) -> u32 {
        (item.value.size() + key.len()) as u32
    }
}
