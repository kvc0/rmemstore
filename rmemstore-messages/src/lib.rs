#[rustfmt::skip]
#[allow(clippy::unwrap_used)]
mod rmemstore;
pub mod protosocket_adapter;

// While I don't normally condone wildcard imports, this is a generated file in a
// package that is made just for this purpose.
pub use rmemstore::*;
