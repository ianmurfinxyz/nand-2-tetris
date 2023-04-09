use std::io::{self, BufRead, Write};
use std::collections::hash_map::HashMap;
use crate::parser::*;
use crate::encoder::*;

mod parser;
mod encoder;

fn write_parse_error(e: &ParseError) {
}

fn write_ram_exhausted_error() {
}

fn write_rom_exhausted_error() {
}

fn assemble<R: ?Sized, W: ?Sized>(asm_in: &mut R, bin_out: &mut W) -> io::Result<(u32, u16)>
	where R: BufRead, W: Write
{
	const MAX_PARSE_ERRORS: u32 = 10;

	let mut sym_key_table = HashMap::new();
	let mut sym_val_table = vec![];

	let mut error_count = 0u32;
	let mut line_count = 0u32;

	let mut next_var_ram_address = 0u16;
	let mut ins_ptr = 0u16;

	// Populate symbol table with base set of values...

	for i in 0..=15 {
		sym_key_table.insert(format!("R{}", i), sym_val_table.len());
		sym_val_table.push((next_var_ram_address, SymUse::ARAM));
		next_var_ram_address += 1;
	}

	for (ram_address, sym) in ["SP", "LCL", "ARG", "THIS", "THAT"].iter().enumerate() {
		sym_key_table.insert(format!("{}", sym), sym_val_table.len());
		sym_val_table.push((ram_address as u16, SymUse::ARAM));
	}

	const SCR_RAM_ADDRESS: u16 = 16384u16;
	const KBD_RAM_ADDRESS: u16 = 24576u16;
	const MAX_ROM_ADDRESS: u16 = 32767u16; // 32Kib

	sym_key_table.insert("SCREEN".to_string(), sym_val_table.len());
	sym_val_table.push((SCR_RAM_ADDRESS, SymUse::ARAM));

	sym_key_table.insert("KBD".to_string(), sym_val_table.len());
	sym_val_table.push((KBD_RAM_ADDRESS, SymUse::ARAM));

	// Parse all instructions into memory...

	let mut inss = vec![];
	for line in asm_in.lines() {
		match parse_ins(&line?, ins_ptr, &mut sym_key_table, &mut sym_val_table){
			Ok(Some(ins)) => {
				inss.push(ins);
				ins_ptr += 1;
			},
			Ok(None) => {
				continue; // skip comment and whitespace lines
			},
			Err(e) => {
				write_parse_error(&e);
				error_count += 1;
				ins_ptr += 1;
				if error_count >= MAX_PARSE_ERRORS {
					return Ok((line_count, ins_ptr));
				}
			},
		}
		if ins_ptr >= MAX_ROM_ADDRESS {
			write_rom_exhausted_error();
			return Ok((line_count, ins_ptr));
		}
		line_count += 1;
	}

	// Distribute RAM addresses to variables...

	for (ram_address, usage) in &mut sym_val_table {
		if *usage == SymUse::ARAM && *ram_address == DEFAULT_RAM_ADDRESS {
			*ram_address = next_var_ram_address;
			next_var_ram_address += 1;
		}
		if next_var_ram_address >= SCR_RAM_ADDRESS {
			write_ram_exhausted_error();
			return Ok((line_count, ins_ptr));
		}
	}

	// Encode instructions and write to disk...

	for ins in inss {
		if let Some(bin_ins) = encode_ins(&ins, &sym_val_table) {
			writeln!(bin_out, "{:016b}", bin_ins)?;
		}
	}

	Ok((line_count, ins_ptr))
}

fn main(){
}
