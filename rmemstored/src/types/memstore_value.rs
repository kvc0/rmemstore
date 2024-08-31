use ahash::HashMap;
use bytes::Bytes;
use rmemstore_messages::Map;

use super::ValueError;

#[derive(Clone, Debug)]
pub enum MemstoreValue {
    Blob { value: Bytes },
    String { value: String },
    Map { map: HashMap<String, MemstoreValue> },
}

impl MemstoreValue {
    pub fn size(&self) -> usize {
        match self {
            MemstoreValue::Blob { value } => value.len(),
            MemstoreValue::String { value } => value.len(),
            MemstoreValue::Map { map } => map.iter().map(|(k, v)| k.len() + v.size()).sum(),
        }
    }
}

impl TryFrom<rmemstore_messages::Value> for MemstoreValue {
    type Error = ValueError;

    fn try_from(value: rmemstore_messages::Value) -> Result<Self, Self::Error> {
        match value.kind {
            Some(kind) => Ok(match kind {
                rmemstore_messages::value::Kind::Blob(value) => MemstoreValue::Blob { value },
                rmemstore_messages::value::Kind::String(value) => MemstoreValue::String { value },
                rmemstore_messages::value::Kind::Map(map) => MemstoreValue::Map {
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

impl From<MemstoreValue> for rmemstore_messages::Value {
    fn from(value: MemstoreValue) -> Self {
        rmemstore_messages::Value {
            kind: Some(match value {
                MemstoreValue::Blob { value } => rmemstore_messages::value::Kind::Blob(value),
                MemstoreValue::String { value } => rmemstore_messages::value::Kind::String(value),
                MemstoreValue::Map { map } => rmemstore_messages::value::Kind::Map(Map {
                    map: map.into_iter().map(|(k, v)| (k, v.into())).collect(),
                }),
            }),
        }
    }
}
