use std::{env::var, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    // // Define all necessary configuration symbols for the configured device:
    let chip = esp_metadata_generated::Chip::from_cargo_feature()?;
    chip.define_cfgs();
    esp_metadata_generated::emit_check_cfg_directives();

    // Fixes some rust-analyzer issues
    println!("cargo::rustc-check-cfg=cfg(rust_analyzer)");

    if cfg!(feature = "defmt") {
        println!("cargo::rustc-link-arg=-Tdefmt.x");
    }

    println!("cargo::rustc-link-arg-tests=-Tembedded-test.x");

    // Link against variables loaded by load_lp_shared! macro.
    println!(
        "cargo::rustc-link-arg=-T{}/lp_shared.x",
        var("OUT_DIR")?.to_string()
    );

    Ok(())
}
