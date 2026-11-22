#![allow(unused)]
use std::{
    env,
    error::Error,
    fmt::Write as _,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use cargo_metadata::Message;
use esp_devtool::{self, cargo::CargoArgsBuilder};
use esp_metadata_generated::Chip;

fn main() -> Result<(), Box<dyn Error>> {
    // Determine the name of the configured device:
    let chip = Chip::from_cargo_feature()?;
    // Define all necessary configuration symbols for the configured device:
    chip.define_cfgs();

    // Called when this crate is built as a library
    #[cfg(not(feature = "is-lp-core"))]
    generate_library_artifacts(chip)?;

    // Done!
    Ok(())
}

fn generate_library_artifacts(chip: Chip) -> Result<(), Box<dyn Error>> {
    // Generated artifacts are placed in $OUT_DIR
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Compile the LP firmware, and return a path to the binary ELF artifact
    let bin = build_lp_firmware(chip)?;

    // Extract symbols from the compiled binary, so the HP core can link against them.
    let linker_script_filename = format!("{}-shared.x", env!("CARGO_PKG_NAME"));
    generate_linker_script(chip, &bin, &out_dir, linker_script_filename)?;

    // The load! macro provides a wrapper around `load_lp_code!`,
    // pre-filled with an absolute path to the firmware ELF.
    generate_load_macro(out_dir, bin)
}

fn generate_load_macro(out_dir: PathBuf, elf_file: PathBuf) -> Result<(), Box<dyn Error>> {
    let mut source = String::from(
        r#"#[doc(hidden)]
#[macro_export]
/// Wrapper for load_lp_code!(...)
macro_rules! load {
"#,
    );

    write!(
        source,
        r#"
    () => {{ load_lp_code!("{elf}") }};
"#,
        elf = elf_file.display()
    )?;

    write!(
        source,
        r#"
}}
"#
    )?;

    std::fs::write(
        std::path::PathBuf::from(out_dir).join("load_macro.rs"),
        source,
    )?;

    Ok(())
}

/// Builds this crate, and returns a path to the compiled ELF.
fn build_lp_firmware(chip: Chip) -> Result<PathBuf, Box<dyn Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_PATH"));
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cwd = manifest_dir; // working directory for cargo command execution

    let pkg_name = env!("CARGO_PKG_NAME");
    let chip_target = match &chip {
        Chip::Esp32c6 => "riscv32imac-unknown-none-elf",
        Chip::Esp32s2 | Chip::Esp32s3 => "riscv32imc-unknown-none-elf",
        _ => panic!("Unsupported chip!"),
    };
    let mut features: Vec<String> = vec![];
    features.push(chip.name().into());
    features.push("is-lp-core".into());

    fn run_build_cmd(
        args: &Vec<String>,
        cwd: &PathBuf,
        capture: bool,
    ) -> Result<std::process::Output, std::io::Error> {
        // Build a 'clean' env variable set, which doesnt
        // inherit any conflicting CARGO/RUSTFLAGS from the parent crate.
        let envs: Vec<(String, String)> = std::env::vars()
            .filter(|(k, _)| !(k.starts_with("CARGO") || k.starts_with("RUST")))
            .collect();

        let mut command = Command::new(env!("CARGO"));
        command
            .args(args)
            .current_dir(cwd)
            .env_clear()
            .envs(envs)
            .stdout(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            })
            .stderr(if capture {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });

        // retrying - remove once we don't need this anymore
        command.stdin(Stdio::inherit()).output()
    }

    let mut builder = CargoArgsBuilder::new(pkg_name.to_string())
        .subcommand("build")
        .manifest_path(manifest_path)
        .target(chip_target)
        .features(&features);
    builder.add_arg("--release");
    // Add this, so that we can read stdout
    // using cargo_metadata, to find the compiled artifacts.
    builder.add_arg("--message-format=json-render-diagnostics");
    let args = builder.build();
    let output = run_build_cmd(&args, &cwd, true).map_err(|e| Box::new(e))?;

    let reader = std::io::BufReader::new(output.stdout.as_slice());
    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerArtifact(artifact) => {
                if let Some(bin) = artifact.executable {
                    println!("{:?}", bin);
                    // Return the first binary file we find!
                    return Ok(PathBuf::from(bin));
                }
            }
            _ => (), // Unknown message
        }
    }

    panic!("Couldn't fetch LP binary!");
}

fn generate_linker_script(
    chip: Chip,
    elf_file: &PathBuf,
    out_dir: &PathBuf,
    filename: String,
) -> Result<(), Box<dyn Error>> {
    type SharedSymbol = (String, u64);

    fn read_shared_variables(elf_file: &PathBuf) -> Result<Vec<SharedSymbol>, Box<dyn Error>> {
        use object::{File, Object, ObjectSymbol};
        let bin_data = std::fs::read(elf_file)?;
        let obj_file = File::parse(&*bin_data)?;
        let syms: Vec<(String, u64)> = obj_file
            .symbols()
            .filter(|s| s.name().unwrap().starts_with("__ULP_$"))
            .map(|s| {
                let full_name = s.name().unwrap();
                let symbol_name: String = full_name.split("$").skip(1).take(1).collect();
                (symbol_name, s.address())
            })
            .collect();

        Ok(syms)
    }

    // Find the shared variable symbols in the ELF binary file
    let symbols = read_shared_variables(&elf_file)?;

    // Determine the base address to offset from HP core to LP core.
    let lp_base_address = match chip {
        Chip::Esp32c6 => 0x0,
        Chip::Esp32s2 | Chip::Esp32s3 => 0x5000_0000,
        _ => panic!("Unsupported chip!"),
    };

    // Write generated symbols to the file
    let link_path = out_dir.join(filename);
    let mut link_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(link_path)?;

    for (s, a) in symbols {
        let offset_addr = a + lp_base_address;
        let entry = format!("PROVIDE({s} = {offset_addr:#x});\r\n");
        link_file.write_fmt(format_args!("{}", entry))?;
        println!("cargo::warning={}", entry);
    }

    Ok(())
}
