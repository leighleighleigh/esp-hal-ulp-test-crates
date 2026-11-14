use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // // Define all necessary configuration symbols for the configured device:
    let chip = esp_metadata_generated::Chip::from_cargo_feature()?;
    chip.define_cfgs();
    esp_metadata_generated::emit_check_cfg_directives();

    // Fixes some rust-analyzer issues
    println!("cargo::rustc-check-cfg=cfg(rust_analyzer)");

    if cfg!(feature = "defmt") {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }

    println!("cargo:rustc-link-arg-tests=-Tembedded-test.x");

    // Built ULP artifacts (ULP ELF binaries and the generated linker script containing shared
    // variable locations), should have been copied into the ./target/ directory.
    println!("cargo::rerun-if-changed=target/lp_app");
    println!("cargo::rerun-if-changed=target/lp_app.x");
    println!("cargo::rustc-link-arg=-Ttarget/lp_app.x");

    println!("cargo::rerun-if-changed=target/lp_rainbow");
    println!("cargo::rerun-if-changed=target/lp_rainbow.x");
    println!("cargo::rustc-link-arg=-Ttarget/lp_rainbow.x");

    Ok(())
}
