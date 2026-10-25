use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Define all necessary configuration symbols for the configured device:
    esp_metadata_generated::Chip::from_cargo_feature()?.define_cfgs();

    // Fixes some rust-analyzer issues
    println!("cargo::rustc-check-cfg=cfg(rust_analyzer)");

    // Include the lp app linker scripts
    println!("cargo::rustc-link-arg=-Tlp_app.x");
    println!("cargo::rustc-link-arg=-Tlp_rainbow.x");

    Ok(())
}
