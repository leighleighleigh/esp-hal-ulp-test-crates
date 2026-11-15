/// Extracts global variables from an ELF file,
/// so that they can be provided in linker scripts.
/// e.g. ULP-core ELF will share symbols, to the HP-core program.
extern crate clap;
extern crate comfy_table;
use std::{error, error::Error, fmt, fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use clap_num::maybe_hex;
use comfy_table::{Cell, Table};
use object::{Object, ObjectSymbol};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input ELF file
    #[arg(short, long)]
    file_name: String,

    // Symbol filtering mode
    #[arg(value_enum, short, long, default_value_t = Mode::V2)]
    mode: Mode,

    /// Offset added to each symbol address
    #[arg(short,long,value_parser=maybe_hex::<u64>)]
    base_address: Option<u64>,

    /// Print using linker script format
    #[arg(short, long)]
    linker_format: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    V1, // Legacy, hard-coded rules, doesnt expect __ULP_ prefix on shared variables.
    V2, // New, expects __ULP_ prefix on shared variables, will remap their names
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let addr_offset = match args.base_address {
        Some(o) => o,
        None => 0,
    };

    let path: PathBuf = From::from(args.file_name);
    let file_data = fs::read(path.clone())?;
    let obj_file = object::File::parse(&*file_data)?;

    let mut global_syms: Vec<_> = obj_file
        .symbols()
        .filter(|x| x.is_global() && (x.kind() == object::SymbolKind::Data))
        .collect::<Vec<_>>();

    // Check for __ULP_MAGIC_..., which confirms that the ELF is a valid ULP/LP program.
    if global_syms
        .iter()
        .filter(|x| x.name().unwrap().starts_with(ULP_MAGIC_PREFIX))
        .count()
        == 0
    {
        return Err(NoMagic.to_string().into());
    }

    // Filter symbols by name, depending on which CLI args were provided
    global_syms = global_syms
        .into_iter()
        .filter(|x| {
            // Remove the ULP magic symbol
            if x.name().unwrap().starts_with(ULP_MAGIC_PREFIX) {
                return false;
            }

            // Different filtering modes can be selected, based on the desired behaviour,
            // this is done to preserve backwards compatibility (functionally, but the CLI interface
            // is not backwards-compat lol)
            match &args.mode {
                Mode::V1 => {
                    if x.name().unwrap().starts_with("_") {
                        return false;
                    }
                    if x.name().unwrap() == "DEVICE_PERIPHERALS" {
                        return false;
                    }
                    true
                }
                Mode::V2 => x.name().unwrap().starts_with(ULP_SHARED_PREFIX),
            }
        })
        .collect();

    // Sort ascending by symbol address
    global_syms.sort_by(|a, b| a.address().partial_cmp(&b.address()).expect("comparison"));

    if !args.linker_format {
        // Pretty-print as a formatted table to stdout
        let mut table = Table::new();
        table.set_header(["input_addr", "input_name", "output_addr", "output_name"]);

        for sym in global_syms.iter() {
            let i_addr = sym.address();
            let i_name = sym.name()?;
            let o_addr = i_addr + addr_offset;
            let o_name = extract_clean_symbol_name(&args.mode, i_name.to_string());

            let cells: Vec<Cell> = vec![
                format!("{:#x}", i_addr).into(),
                i_name.into(),
                format!("{:#x}", o_addr).into(),
                o_name.into(),
            ];
            table.add_row(cells);
        }
        println!("{table}");
    } else {
        // Print as a linker script to stdout
        for sym in global_syms.iter() {
            let i_addr = sym.address();
            let i_name = sym.name()?;
            let o_addr = i_addr + addr_offset;
            let o_name = extract_clean_symbol_name(&args.mode, i_name.to_string());

            // Different entries per symbol, depending on mode
            match &args.mode {
                Mode::V1 => {
                    println!("PROVIDE({} = {});", o_name, format!("{:#x}", o_addr));
                }
                Mode::V2 => {
                    println!("PROVIDE({} = {});", i_name, format!("{:#x}", o_addr));
                    println!("PROVIDE({} = {});", o_name, i_name);
                }
            }
        }
    }

    Ok(())
}

fn extract_clean_symbol_name(mode: &Mode, iname: String) -> String {
    match mode {
        Mode::V1 => iname,
        Mode::V2 => iname
            .trim_start_matches(ULP_SHARED_PREFIX)
            .splitn(2, "$")
            .take(1)
            .collect(),
    }
}

const ULP_MAGIC_PREFIX: &str = "__ULP_MAGIC_";
const ULP_SHARED_PREFIX: &str = "__ULP_$";

#[derive(Debug, Clone)]
struct NoMagic;

impl fmt::Display for NoMagic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Given file doesn't seem to be an LP/ULP core application."
        )
    }
}

impl error::Error for NoMagic {}
