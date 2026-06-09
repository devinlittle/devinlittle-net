use std::{env, path::PathBuf};

use anyhow::Error;

fn main() -> Result<(), Error> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("mesh.service_connector.bin"))
        .compile_protos(&["proto/service_connector.proto"], &["proto"])?;
    Ok(())
}
