fn main() {
    tonic_build::configure()
        .build_server(false)
        .compile_protos(
            &["proto/kaiko/benchmark_reference_rates.proto"],
            &["proto/"],
        )
        .expect("Failed to compile .proto")
}
