fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["proto/kaiko/streaming.proto"], &["proto/"])
        .expect("Failed to compile .proto")
}
