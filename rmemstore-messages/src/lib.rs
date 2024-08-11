#[rustfmt::skip]
#[allow(clippy::unwrap_used)]
mod rmemstore;

// While I don't normally condone wildcard imports, this is a generated file in a
// package that is made just for this purpose.
pub use rmemstore::*;
