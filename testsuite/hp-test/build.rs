use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Define all necessary configuration symbols for the configured device:
    esp_metadata_generated::Chip::from_cargo_feature()?.define_cfgs();

    // Include the lp app x file
    // Print out the extra argument
    println!("cargo:rustc-link-arg=-Tlp_app.x");

    Ok(())
}
