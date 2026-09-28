use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Define all necessary configuration symbols for the configured device:
    esp_metadata_generated::Chip::from_cargo_feature()?.define_cfgs();

    println!("cargo::rustc-check-cfg=cfg(rust_analyzer)");
    println!("cargo::rustc-link-arg=-Tembedded-test.x");
    println!("cargo::rustc-link-arg=-Tdefmt.x");

    // Include the lp app x files
    println!("cargo::rustc-link-arg=-Tlp_app.x");
    println!("cargo::rustc-link-arg=-Tlp_rainbow.x");

    Ok(())
}
