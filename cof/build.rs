use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .build_transport(false)
        .build_client(false)
        .build_server(false)
        .compile_protos(&["cof/common/dice/v1/dice.proto"], &["../proto"])?;

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("diceapiv1_descriptor.bin"))
        .build_transport(true)
        .build_client(true)
        .build_server(true)
        .compile_protos(&["cof/dice_api/v1/service.proto"], &["../proto"])?;

    Ok(())
}
