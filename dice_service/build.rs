use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=repo/postgres/migrations");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Cannot get OUT_DIR env var"));

    tonic_prost_build::configure()
        .file_descriptor_set_path(out_dir.join("diceapiv1_descriptor.bin"))
        .build_transport(true)
        .build_client(true)
        .build_server(true)
        .compile_protos(&["cof/dice_api/v1/service.proto"], &["../proto"])?;

    Ok(())
}
