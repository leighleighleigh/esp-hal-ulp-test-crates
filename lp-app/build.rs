#![allow(unused)]
use std::{
    env,
    error::Error,
    io::{self, Write},
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
    #[cfg(not(feature = "is-lp-core"))]
    build_lp_app(chip)?;
    // Done!
    Ok(())
}

fn build_lp_app(chip: Chip) -> Result<(), Box<dyn Error>> {
    let bin = fetch_lp_binary(chip)?;
    let syms = read_shared_variables(&bin)?;
    link_shared_variables(syms)
}

fn fetch_lp_binary(chip: Chip) -> Result<PathBuf, Box<dyn Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_PATH"));
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let pkg_name = env!("CARGO_PKG_NAME");

    let chip_target = match &chip {
        Chip::Esp32c6 => "riscv32imac-unknown-none-elf",
        Chip::Esp32s2 | Chip::Esp32s3 => "riscv32imc-unknown-none-elf",
        _ => panic!("Unsupported chip!"),
    };

    let mut features: Vec<String> = vec![];
    features.push(chip.name().into());
    features.push("is-lp-core".into());

    let cwd = manifest_dir.to_path_buf();
    let mut builder = CargoArgsBuilder::new(pkg_name.to_string())
        .subcommand("build")
        .manifest_path(cwd.clone().join("Cargo.toml"))
        .target(chip_target)
        .features(&features);
    builder.add_arg("--release");
    // Add this, so that we can read stdout
    // using cargo_metadata, to find the compiled artifacts.
    builder.add_arg("--message-format=json-render-diagnostics");

    let args = builder.build();
    // println!("cargo::warning={:?}", args);
    let output = run_build_cmd(&args, &cwd, true).map_err(|e| Box::new(e))?;
    // println!("cargo::warning={:?}", output);

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

fn link_shared_variables(symbols: Vec<SharedSymbol>) -> Result<(), Box<dyn Error>> {
    // Generate a new linker script, containing the shared variable locations
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Unsure if needed, if we aren't ourselves linking against this? Hmm, need do test.
    println!("cargo:rustc-link-search={}", out.display());
    let link_path = out.join(format!("{}-shared.x", env!("CARGO_PKG_NAME")));
    let mut link_file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(link_path)?;
    // Write to the file
    for (s, a) in symbols {
        let offset_addr = a + 0x50000000;
        let entry = format!("PROVIDE({s} = {offset_addr:#x});");
        println!("cargo::warning={}", entry);
        link_file.write_fmt(format_args!("{}", entry))?;
    }
    Ok(())
}
