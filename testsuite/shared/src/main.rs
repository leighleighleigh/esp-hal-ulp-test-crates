/// Extracts global variables from an ELF file,
/// so that they can be provided in linker scripts.
/// e.g. ULP-core ELF will share symbols, to the HP-core program.
extern crate clap;
extern crate comfy_table;
extern crate elf;

use clap::Parser;
use clap_num::maybe_hex;
use comfy_table::{Cell, Table};
use elf::endian::AnyEndian;
// use elf::note::Note;
// use elf::relocation::{RelIterator, RelaIterator};
// use elf::to_str::{e_machine_to_human_str, e_osabi_to_string, e_type_to_human_str, st_symtype_to_str};
use elf::ElfStream;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    file_name: String,

    #[arg(long)]
    show_zero_sized: bool,

    #[arg(short,long)]
    linker : bool,

    #[arg(short,long,value_parser=maybe_hex::<u64>)]
    base_address : Option<u64>,
}

// Represents an extracted STB_GLOBAL STT_OBJECT symbol,
// with fully qualified address
#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct GlobalObjectSymbol {
    address: u64,
    name: String,
}

fn get_global_symbols(elf_file: &mut ElfStream<AnyEndian, std::fs::File>, allow_zero_sized : bool) -> Vec<GlobalObjectSymbol>
{
    let elf_section_hdrs= elf_file.section_headers().clone();
    let elf_symtab = elf_file.symbol_table().expect("got .symtab").clone();

    let (symtab, strtab) = match elf_symtab
    {
        Some(tables) => tables,
        None => return vec![],
    };

    // Filter the symbols, then extract their address and name
    symtab.iter().filter(|sym| {
        (sym.st_bind() == elf::abi::STB_GLOBAL)
            &&
        (sym.st_symtype() == elf::abi::STT_OBJECT)
            &&
        (allow_zero_sized || (sym.st_size != 0))
    }).map(|sym| {
        // symbol name is stored in strtab
        let name = strtab.get(sym.st_name as usize).expect("got symbol name");
        // resolve the symbol address relative to it's section offset address
        let sym_section = elf_section_hdrs[sym.st_shndx as usize];
        let symbol_address = sym_section.sh_offset + sym.st_value;
        
        GlobalObjectSymbol { address: symbol_address, name: name.into() }
    }).collect()
}

fn print_global_symbols(elf_file: &mut ElfStream<AnyEndian, std::fs::File>, base_address : u64, allow_zero_sized : bool) {
    let global_syms = get_global_symbols(elf_file, allow_zero_sized);
    let mut table = Table::new();
    // if plain {
    //     table.load_preset(comfy_table::presets::NOTHING);
    // }
    table.set_header([
        "sym_addr",
        "sym_name",
    ]);
    for sym in global_syms.iter() {
        let cells: Vec<Cell> = vec![
            format!("{:#x}", sym.address + base_address).into(),
            sym.name.clone().into(),
        ];
        table.add_row(cells);
    }
    println!("{table}");
}


fn print_global_symbols_as_linkerscript(elf_file: &mut ElfStream<AnyEndian, std::fs::File>, base_address : u64, allow_zero_sized : bool) {
    let global_syms = get_global_symbols(elf_file, allow_zero_sized);

    for sym in global_syms.iter() {
        println!("PROVIDE({} = {});",sym.name.clone(),format!("{:#x}",sym.address + base_address));
        // format!("{:#x}", sym.address)
        // sym.name.clone()
    }
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

    if !args.linker {
        print_global_symbols(&mut elf_file, addr_offset, args.show_zero_sized);
    } else {
        print_global_symbols_as_linkerscript(&mut elf_file, addr_offset, args.show_zero_sized);
    }
}
