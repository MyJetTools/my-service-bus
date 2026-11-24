fn main() {
    // Compiled from the synced copy in `proto/` rather than downloaded, as my-service-bus-persistence
    // does: ci-utils 0.1.3 does not build against fl-url 0.7.0 (it still calls the removed
    // `FlUrlResponse::receive_body`), so a clean build - one without a cargo cache that still
    // holds the old fl-url - fails in the build script. Switch back to
    // `ci_utils::sync_and_build_proto_file` once ci-utils is fixed - the contract still lives in
    // the my-sb-proto-files repo.
    println!("cargo:rerun-if-changed=proto/MyServicePersistenceGrpcService.proto");
    ci_utils::compile_protos("proto/MyServicePersistenceGrpcService.proto");
}
