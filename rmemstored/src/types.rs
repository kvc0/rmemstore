use ahash::HashMap;
use bytes::Bytes;
use messages::rmemstore::Map;

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
    Map { map: HashMap<String, MemstoreValue> },
}

impl MemstoreValue {
    pub fn size(&self) -> usize {
        match self {
            MemstoreValue::Blob { value } => value.len(),
            MemstoreValue::Map { map } => map.iter().map(|(k, v)| k.len() + v.size()).sum(),
        }
    }
}

impl MemstoreItem {
    pub fn weigher(key: &Bytes, item: &MemstoreItem) -> u32 {
        (item.value.size() + key.len()) as u32
    }
}

#[derive(Clone, Debug)]
pub struct MemstoreWeigher;
impl k_cache::Weigher<Bytes, MemstoreItem> for MemstoreWeigher {
    fn weigh(key: &Bytes, item: &MemstoreItem) -> usize {
        MemstoreItem::weigher(key, item) as usize
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ValueError {
    #[error("Missing attribute: {0}")]
    MissingAttribute(&'static str),
}

impl TryFrom<messages::rmemstore::Value> for MemstoreValue {
    type Error = ValueError;

    fn try_from(value: messages::rmemstore::Value) -> Result<Self, Self::Error> {
        match value.kind {
            Some(kind) => Ok(match kind {
                messages::rmemstore::value::Kind::Blob(value) => MemstoreValue::Blob { value },
                messages::rmemstore::value::Kind::Map(map) => MemstoreValue::Map {
                    map: map
                        .map
                        .into_iter()
                        .map(|(k, v)| TryInto::<MemstoreValue>::try_into(v).map(|v| (k, v)))
                        .collect::<Result<_, Self::Error>>()?,
                },
            }),
            None => Err(ValueError::MissingAttribute("Value kind")),
        }
    }
}

impl From<MemstoreValue> for messages::rmemstore::Value {
    fn from(value: MemstoreValue) -> Self {
        messages::rmemstore::Value {
            kind: Some(match value {
                MemstoreValue::Blob { value } => messages::rmemstore::value::Kind::Blob(value),
                MemstoreValue::Map { map } => messages::rmemstore::value::Kind::Map(Map {
                    map: map.into_iter().map(|(k, v)| (k, v.into())).collect(),
                }),
            }),
        }
    }
}
