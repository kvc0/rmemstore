use std::collections::HashMap;

use bytes::Bytes;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum MemstoreValue {
    Blob { value: Bytes },
    String { string: String },
    Map { map: HashMap<String, MemstoreValue> },
}

pub trait IntoKey {
    fn into_key(self) -> Bytes;
}

pub trait IntoValue {
    fn into_value(self) -> rmemstore_messages::value::Kind;
}

impl IntoKey for Bytes {
    fn into_key(self) -> Bytes {
        self
    }
}

impl IntoKey for Vec<u8> {
    fn into_key(self) -> Bytes {
        self.into()
    }
}

impl IntoKey for &[u8] {
    fn into_key(self) -> Bytes {
        Bytes::copy_from_slice(self)
    }
}

impl IntoKey for &str {
    fn into_key(self) -> Bytes {
        Bytes::copy_from_slice(self.as_bytes())
    }
}

impl IntoKey for String {
    fn into_key(self) -> Bytes {
        self.into_bytes().into_key()
    }
}

impl IntoValue for Bytes {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::Blob(self)
    }
}

impl IntoValue for Vec<u8> {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::Blob(self.into())
    }
}

impl IntoValue for &[u8] {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::Blob(Bytes::copy_from_slice(self))
    }
}

impl IntoValue for &str {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::String(self.to_owned())
    }
}

impl IntoValue for String {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::String(self)
    }
}

impl<K, V, S> IntoValue for HashMap<K, V, S>
where
    K: Into<String>,
    V: IntoValue,
{
    fn into_value(self) -> rmemstore_messages::value::Kind {
        rmemstore_messages::value::Kind::Map(rmemstore_messages::Map {
            map: self
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.into(),
                        rmemstore_messages::Value {
                            kind: Some(v.into_value()),
                        },
                    )
                })
                .collect(),
        })
    }
}

impl IntoValue for rmemstore_messages::value::Kind {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        self
    }
}

impl IntoValue for MemstoreValue {
    fn into_value(self) -> rmemstore_messages::value::Kind {
        match self {
            MemstoreValue::Blob { value } => value.into_value(),
            MemstoreValue::String { string: value } => value.into_value(),
            MemstoreValue::Map { map } => {
                rmemstore_messages::value::Kind::Map(rmemstore_messages::Map {
                    map: map
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                k,
                                rmemstore_messages::Value {
                                    kind: Some(v.into_value()),
                                },
                            )
                        })
                        .collect(),
                })
            }
        }
    }
}

impl TryFrom<rmemstore_messages::Value> for MemstoreValue {
    type Error = crate::Error;

    fn try_from(value: rmemstore_messages::Value) -> Result<Self, Self::Error> {
        match value.kind {
            Some(kind) => match kind {
                rmemstore_messages::value::Kind::Blob(value) => Ok(Self::Blob { value }),
                rmemstore_messages::value::Kind::String(value) => {
                    Ok(Self::String { string: value })
                }
                rmemstore_messages::value::Kind::Map(rmemstore_messages::Map { map }) => {
                    Ok(Self::Map {
                        map: map
                            .into_iter()
                            .map(|(k, v)| v.try_into().map(|v| (k, v)))
                            .collect::<Result<_, crate::Error>>()?,
                    })
                }
            },
            None => Err(crate::Error::MalformedResponse("missing value kind")),
        }
    }
}
