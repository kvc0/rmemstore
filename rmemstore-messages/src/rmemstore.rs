// This file is @generated by prost-build.
/// An action to be taken
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Rpc {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(uint32, tag = "2")]
    pub code: u32,
    #[prost(oneof = "rpc::Command", tags = "3, 4")]
    pub command: ::core::option::Option<rpc::Command>,
}
/// Nested message and enum types in `Rpc`.
pub mod rpc {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        /// Response kind: ok
        #[prost(message, tag = "3")]
        Put(super::Put),
        /// Response kind: Value
        #[prost(message, tag = "4")]
        Get(super::Get),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(uint64, tag = "1")]
    pub id: u64,
    #[prost(uint32, tag = "2")]
    pub code: u32,
    #[prost(oneof = "response::Kind", tags = "3, 4")]
    pub kind: ::core::option::Option<response::Kind>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(bool, tag = "3")]
        Ok(bool),
        #[prost(message, tag = "4")]
        Value(super::Value),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    #[prost(oneof = "value::Kind", tags = "1, 2, 3")]
    pub kind: ::core::option::Option<value::Kind>,
}
/// Nested message and enum types in `Value`.
pub mod value {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        #[prost(bytes, tag = "1")]
        Blob(::prost::bytes::Bytes),
        #[prost(string, tag = "2")]
        String(::prost::alloc::string::String),
        #[prost(message, tag = "3")]
        Map(super::Map),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Map {
    #[prost(map = "string, message", tag = "1")]
    pub map: ::std::collections::HashMap<::prost::alloc::string::String, Value>,
}
/// Returns response.kind.ok
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Put {
    #[prost(bytes = "bytes", tag = "1")]
    pub key: ::prost::bytes::Bytes,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<Value>,
}
/// Returns response.kind.value, or no value upon a miss.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Get {
    #[prost(bytes = "bytes", tag = "1")]
    pub key: ::prost::bytes::Bytes,
}
