use std::{env::var, error::Error, fs::File, io::Write, path::PathBuf};

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

    // Built ULP artifacts (ULP ELF binaries and the generated linker script containing shared
    // variable locations), should have been copied into the ./target/ directory.
    println!("cargo::rerun-if-changed=target/lp_app");
    println!("cargo::rerun-if-changed=target/lp_rainbow");

    // build_lp_shared_linker_script()?;

    let out_dir = var("OUT_DIR")?;
    println!("cargo::rustc-link-search={}", out_dir.to_string());
    println!("cargo::rustc-link-arg=-Tlp_shared.x");

    // // Alternative to linker script generation, this also works, but would require the build
    // script // to load the LP application itself (probably).
    // println!("cargo::rustc-link-arg=-Wl,--defsym=RAINBOW_RUN=0x500008fc");
    // println!("cargo::rustc-link-arg=-Wl,--defsym=RAINBOW_COUNTER=0x500008f8");

    Ok(())
}

#[allow(unused)]
fn build_lp_shared_linker_script() -> Result<(), Box<dyn Error>> {
    let out_dir = var("OUT_DIR")?;
    println!("cargo::rustc-link-search={}", out_dir.to_string());

    let xpath = PathBuf::from(&(out_dir.clone() + "/lp_shared.x"));
    let mut outx = File::create(xpath)?;
    outx.write_fmt(format_args!("{}", LP_APP_X))?;
    outx.flush()?;
    drop(outx);
    println!("cargo::rustc-link-arg=-Tlp_shared.x");

    Ok(())
}

const LP_APP_X: &str = r#"PROVIDE(ULP_TEST_DATA_OUT = __ULP_$ULP_TEST_DATA_OUT$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_TEST_DATA_IN = __ULP_$ULP_TEST_DATA_IN$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_REPLY = __ULP_$ULP_REPLY$7368617265643a3a556c705265706c7957726170706572$);
PROVIDE(ULP_LOOP_COUNTER = __ULP_$ULP_LOOP_COUNTER$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_LOCK = __ULP_$ULP_LOCK$7368617265643a3a556c704c6f636b57726170706572$);
PROVIDE(ULP_HALT_COUNTER = __ULP_$ULP_HALT_COUNTER$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_DEBUG_LAST_ISR_DATA = __ULP_$ULP_DEBUG_LAST_ISR_DATA$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_DEBUG_GPIO_ISR_STATUS = __ULP_$ULP_DEBUG_GPIO_ISR_STATUS$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_DEBUG_GPIO_ISR_COUNT = __ULP_$ULP_DEBUG_GPIO_ISR_COUNT$7368617265643a3a556c70436f756e746572$);
PROVIDE(ULP_COMMAND = __ULP_$ULP_COMMAND$7368617265643a3a556c70436d6457726170706572$);
PROVIDE(ULP_BOOT_COUNTER = __ULP_$ULP_BOOT_COUNTER$7368617265643a3a556c70436f756e746572$);
PROVIDE(__ULP_$ULP_TEST_DATA_OUT$7368617265643a3a556c70436f756e746572$ = 0x500010d0);
PROVIDE(__ULP_$ULP_TEST_DATA_IN$7368617265643a3a556c70436f756e746572$ = 0x500010cc);
PROVIDE(__ULP_$ULP_REPLY$7368617265643a3a556c705265706c7957726170706572$ = 0x500010c8);
PROVIDE(__ULP_$ULP_LOOP_COUNTER$7368617265643a3a556c70436f756e746572$ = 0x500010c4);
PROVIDE(__ULP_$ULP_LOCK$7368617265643a3a556c704c6f636b57726170706572$ = 0x500010c0);
PROVIDE(__ULP_$ULP_HALT_COUNTER$7368617265643a3a556c70436f756e746572$ = 0x500010bc);
PROVIDE(__ULP_$ULP_DEBUG_LAST_ISR_DATA$7368617265643a3a556c70436f756e746572$ = 0x500010b8);
PROVIDE(__ULP_$ULP_DEBUG_GPIO_ISR_STATUS$7368617265643a3a556c70436f756e746572$ = 0x500010b4);
PROVIDE(__ULP_$ULP_DEBUG_GPIO_ISR_COUNT$7368617265643a3a556c70436f756e746572$ = 0x500010b0);
PROVIDE(__ULP_$ULP_COMMAND$7368617265643a3a556c70436d6457726170706572$ = 0x500010ac);
PROVIDE(__ULP_$ULP_BOOT_COUNTER$7368617265643a3a556c70436f756e746572$ = 0x500010a8);
"#;
