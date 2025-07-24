use std::process::Command;

fn main() {
    let branch_output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    let git_branch = String::from_utf8(branch_output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);

    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    gen_proto("./src/grpc/proto/hello.proto", "./src/grpc");
    gen_proto("./src/rocksdb/proto/graph.proto", "./src/rocksdb");
}
fn gen_proto(proto_file: &str, out_dir: &str) {
    tonic_build::configure()
        .build_server(true)
        .out_dir(out_dir)
        .compile_protos(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("!!! protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file);
}
