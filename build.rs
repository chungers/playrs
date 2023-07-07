fn main() {
    let proto_file = "./src/grpc/proto/hello.proto";

    println!(">>>>> Building proto file={}", proto_file);

    tonic_build::configure()
        .build_server(true)
        .out_dir("./src/grpc")
        .compile(&[proto_file], &["."])
        .unwrap_or_else(|e| panic!("!!! protobuf compile error: {}", e));

    println!("cargo:rerun-if-changed={}", proto_file);
}
