use std::collections::HashMap;

use bytes::Bytes;
use messages::rmemstore;

pub trait IntoKey {
    fn into_key(self) -> Bytes;
}

pub trait IntoValue {
    fn into_value(self) -> rmemstore::value::Kind;
}

impl IntoKey for Bytes {
    fn into_key(self) -> Bytes {
        self.into()
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
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Blob(self)
    }
}

impl IntoValue for Vec<u8> {
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Blob(self.into())
    }
}

impl IntoValue for &[u8] {
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Blob(Bytes::copy_from_slice(self))
    }
}

impl IntoValue for &str {
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Blob(Bytes::copy_from_slice(self.as_bytes()))
    }
}

impl IntoValue for String {
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Blob(self.into_bytes().into())
    }
}

impl<K, V, S> IntoValue for HashMap<K, V, S>
where
    K: Into<String>,
    V: IntoValue,
{
    fn into_value(self) -> rmemstore::value::Kind {
        rmemstore::value::Kind::Map(rmemstore::Map {
            map: self
                .into_iter()
                .map(|(k, v)| {
                    (
                        k.into(),
                        rmemstore::Value {
                            kind: Some(v.into_value()),
                        },
                    )
                })
                .collect(),
        })
    }
}

impl IntoValue for rmemstore::value::Kind {
    fn into_value(self) -> rmemstore::value::Kind {
        self
    }
}
