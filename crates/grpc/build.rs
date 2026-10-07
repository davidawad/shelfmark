//! Compiles `proto/shelfmark/v1/shelfmark.proto` with protox (no `protoc`).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = "../../proto";
    let file = "shelfmark/v1/shelfmark.proto";
    println!("cargo:rerun-if-changed={root}/{file}");
    let fds = protox::compile([file], [root])?;
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_fds(fds)?;
    Ok(())
}
