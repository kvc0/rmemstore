#[rustfmt::skip]
#[allow(clippy::all)]
mod rmemstore {
    include!(concat!(env!("OUT_DIR"), "/rmemstore.rs"));
}

pub mod protosocket_adapter;

// While I don't normally condone wildcard imports, this is a generated file in a
// package that is made just for this purpose.
pub use rmemstore::*;
