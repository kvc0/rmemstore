#[allow(clippy::unwrap_used)]
fn main() {
    let proto_dir = "../proto";

    prost_build::compile_protos(&["../proto/rmemstore.proto"], &[proto_dir]).unwrap();

    println!("cargo:rerun-if-changed={proto_dir}");
}
