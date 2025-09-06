fn main() {
    // Generate LALRPOP parser
    match lalrpop::process_root() {
        Ok(()) => println!("cargo:rerun-if-changed=src/query_compiler/grammar.lalrpop"),
        Err(e) => {
            eprintln!("LALRPOP failed: {}", e);
            std::process::exit(1);
        }
    }
    
    // Compile protobuf definitions
    if let Err(e) = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/core/distributed/grpc")
        .compile(&["proto/transport.proto"], &["proto"]) {
        eprintln!("Protobuf compilation failed: {}", e);
        // Don't exit here, as gRPC might be optional
    }
}
