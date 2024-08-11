#[allow(clippy::unwrap_used)]
fn main() {
    let proto_dir = "../proto";

    prost_build::Config::new()
        .bytes(&[
            ".rmemstore.Value.blob",
            ".rmemstore.Get.blob",
            ".rmemstore.Put.key",
            ".rmemstore.Get.key",
        ])
        .protoc_arg("--python_out=pyi_out:../example-python")
        .out_dir("./src")
        .compile_protos(&["../proto/rmemstore.proto"], &[proto_dir])
        .unwrap();

    println!("cargo:rerun-if-changed={proto_dir}");
}