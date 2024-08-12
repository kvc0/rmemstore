#[allow(clippy::unwrap_used)]
fn main() {
    let proto_dir = "proto";

    let mut config = prost_build::Config::new();
    config.bytes([
        ".rmemstore.Value.blob",
        ".rmemstore.Get.blob",
        ".rmemstore.Put.key",
        ".rmemstore.Get.key",
    ]);
    config.out_dir("./src");

    if std::env::var("CI").is_ok() {
        eprintln!("skipping python output");
    } else {
        eprintln!("adding python output");
        config.protoc_arg("--python_out=pyi_out:../example-python/");
    }
    config
        .compile_protos(&["./proto/rmemstore.proto"], &[proto_dir])
        .unwrap();

    println!("cargo:rerun-if-changed={proto_dir}");
}
