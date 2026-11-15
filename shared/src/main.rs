/// Extracts global variables from an ELF file,
/// so that they can be provided in linker scripts.
/// e.g. ULP-core ELF will share symbols, to the HP-core program.
extern crate clap;
extern crate comfy_table;
extern crate elf;

use std::{error::Error, fs, path::PathBuf};

use clap::Parser;
use clap_num::maybe_hex;
use comfy_table::{Cell, Table};
// use object::{Object, ObjectSection};
use elf::ElfStream;
use elf::endian::AnyEndian;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,

    #[arg(long)]
    show_zero_sized: bool,

    #[arg(short, long)]
    linker: bool,

    #[arg(short,long,value_parser=maybe_hex::<u64>)]
    base_address: Option<u64>,
}

// Represents an extracted STB_GLOBAL STT_OBJECT symbol,
// with fully qualified address
#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct GlobalObjectSymbol {
    address: u64,
    name: String,
}

fn main() {
    let args = Args::parse();

    let path: PathBuf = From::from(args.file_name);
    let io = std::fs::File::open(path).expect("Could not open file");

    let mut elf_file =
        ElfStream::<AnyEndian, _>::open_stream(io).expect("Failed to open ELF stream");

    let addr_offset = match args.base_address {
        Some(o) => o,
        None => 0,
    };

    let elf_section_hdrs = elf_file.section_headers().clone();
    let elf_symtab = elf_file.symbol_table().expect("got .symtab").clone();
    let mut global_syms = vec![];

    if let Some((symtab, strtab)) = elf_symtab {
        // Filter the symbols, then extract their address and name
        global_syms = symtab
            .iter()
            .filter(|sym| {
                (sym.st_bind() == elf::abi::STB_GLOBAL)
                    && (sym.st_symtype() == elf::abi::STT_OBJECT)
                    && (args.show_zero_sized || (sym.st_size != 0))
            })
            .map(|sym| {
                // symbol name is stored in strtab
                let name = strtab.get(sym.st_name as usize).expect("got symbol name");
                // resolve the symbol address relative to it's section offset address
                let _sym_section = elf_section_hdrs[sym.st_shndx as usize];
                let symbol_address = sym.st_value;
                GlobalObjectSymbol {
                    address: symbol_address,
                    name: name.into(),
                }
            })
            .filter(|gos| !gos.name.starts_with("_") && (gos.name != "DEVICE_PERIPHERALS"))
            .collect();
    };

    if !args.linker {
        // Pretty-print as a formatted table to stdout
        let mut table = Table::new();
        table.set_header(["sym_addr", "sym_name"]);
        for sym in global_syms.iter() {
            let cells: Vec<Cell> = vec![
                format!("{:#x}", sym.address + addr_offset).into(),
                sym.name.clone().into(),
            ];
            table.add_row(cells);
        }
        println!("{table}");
    } else {
        // Print as a linker script to stdout
        for sym in global_syms.iter() {
            println!(
                "PROVIDE({} = {});",
                sym.name.clone(),
                format!("{:#x}", sym.address + addr_offset)
            );
        }
    }
}
