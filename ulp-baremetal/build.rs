// build.rs
use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it.
    // fs::write(out_dir.join("interrupts.x"), include_bytes!("ld/interrupts.x")).unwrap();
    // fs::write(out_dir.join("exceptions.x"), include_bytes!("ld/exceptions.x")).unwrap();
    // fs::write(out_dir.join("memory.x"), include_bytes!("ld/memory.x")).unwrap();
    // fs::write(out_dir.join("link.x"), include_bytes!("ld/link.x")).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    // println!("cargo:rerun-if-changed=ld/interrupts.x");
    // println!("cargo:rerun-if-changed=ld/exceptions.x");
    // println!("cargo:rerun-if-changed=ld/memory.x");
    // println!("cargo:rerun-if-changed=ld/link.x");
    println!("cargo:rerun-if-changed=build.rs");
}
