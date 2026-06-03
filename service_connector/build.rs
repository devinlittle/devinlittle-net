use anyhow::Error;

fn main() -> Result<(), Error> {
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&["proto/service_connector.proto"], &["proto"])?;
    Ok(())
}
