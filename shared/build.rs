use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Fixes some rust-analyzer issues
    println!("cargo::rustc-check-cfg=cfg(rust_analyzer)");
    // Done!
    Ok(())
}
