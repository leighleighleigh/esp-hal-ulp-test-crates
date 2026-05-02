// build.rs
use std::{env, error::Error, path::PathBuf};

use esp_metadata_generated::Chip;

fn main() -> Result<(), Box<dyn Error>> {
    // Determine the name of the configured device:
    let chip = Chip::from_cargo_feature()?;
    // Define all necessary configuration symbols for the configured device:
    chip.define_cfgs();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Put the linker script somewhere the linker can find it.
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    // Done!
    Ok(())
}
