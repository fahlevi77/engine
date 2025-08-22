fn main() {
    lalrpop::process_root().expect("failed to run lalrpop");
    
    // Compile protobuf definitions
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/core/distributed/grpc")
        .compile(&["proto/transport.proto"], &["proto"])
        .expect("Failed to compile protobuf definitions");
}
