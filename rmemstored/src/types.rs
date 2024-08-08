use bytes::Bytes;

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
}

#[derive(Clone, Debug)]
pub enum MemstoreValue {
    Blob { value: Bytes },
}

impl MemstoreItem {
    pub fn weigher(key: &Bytes, item: &MemstoreItem) -> u32 {
        (match &item.value {
            MemstoreValue::Blob { value } => value.len(),
        } + key.len()) as u32
    }
}

impl From<messages::rmemstore::value::Kind> for MemstoreValue {
    fn from(kind: messages::rmemstore::value::Kind) -> Self {
        match kind {
            messages::rmemstore::value::Kind::Blob(value) => MemstoreValue::Blob { value },
        }
    }
}

impl From<MemstoreValue> for messages::rmemstore::response::Kind {
    fn from(value: MemstoreValue) -> Self {
        messages::rmemstore::response::Kind::Value(messages::rmemstore::Value {
            kind: Some(match value {
                MemstoreValue::Blob { value } => messages::rmemstore::value::Kind::Blob(value),
            }),
        })
    }
}
