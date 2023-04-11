use std::io::{self, BufRead, Write};
use std::collections::hash_map::HashMap;
use crate::parser::*;
use crate::encoder::*;

fn write_error(line: &str, line_num: u32, ins_ptr: u16, msg: &str){
	println!("error: {}\n[ip:{},ln:{}] | {}\n", msg, ins_ptr, line_num, line);
}

fn write_pos_error(found: char, pos: usize, line: &str, line_num: u32, ins_ptr: u16, msg: &str){
	let dat = format!("[ip:{},ln:{}] | ", ins_ptr, line_num);
	let pnt = format!("{}{}^", " ".repeat(dat.len()), "~".repeat(pos - 1));
	println!("Unexpected character '{}' at pos '{}'. {}\n{}{}\n{}", found, pos, msg, dat, line, pnt);
}

fn write_parse_error(e: &ParseError, line: &str, line_num: u32, ins_ptr: u16) {
	match e {
		ParseError::UnknownMne{mne_type, mne_buf} => {
			let mne_type_str = match mne_type {
				Some(mt) => format!("{} ", mt.as_str()),
				None => "".to_string(),
			};
			let mne_str = std::str::from_utf8(mne_buf.as_ref()).unwrap().trim();
			let msg = format!("Unknown {}mnemonic '{}'", mne_type_str, mne_str);
			write_error(line, line_num, ins_ptr, &msg);
		},
		ParseError::ExpectedFirstSymChar{found, pos} => {
			write_pos_error(*found, *pos, line, line_num, ins_ptr, "Expected valid first symbol character.");
		},
		ParseError::ExpectedSymChar{found, pos} => {
			write_pos_error(*found, *pos, line, line_num, ins_ptr, "Expected valid symbol character.");
		},
		ParseError::ExpectedDigit{found, pos} => {
			write_pos_error(*found, *pos, line, line_num, ins_ptr, "Expected digit.");
		},
		ParseError::UnexpectedChar{found, pos} => {
			write_pos_error(*found, *pos, line, line_num, ins_ptr, "");
		},
		ParseError::DuplicateLabel => {
			write_error(line, line_num, ins_ptr, "Duplicate label definition!");
		},
		ParseError::AInsMissingArg => {
			write_error(line, line_num, ins_ptr, "Expected argument after opening '@' character for A-instruction.");
		},
		ParseError::LInsMissingSym => {
			write_error(line, line_num, ins_ptr, "Expected symbol after opening '(' character for L-instruction.");
		},
		ParseError::LInsMissingClose => {
			write_error(line, line_num, ins_ptr, "Expected closing ')' character for label.");
		},
		ParseError::SymOverflow => {
			let msg = format!("Symbol too large! Max symbol length is {} characters.", MAX_SYM_LEN);
			write_error(line, line_num, ins_ptr, &msg);
		},
		ParseError::IntOverflow => {
			write_error(line, line_num, ins_ptr, "Integer too large! Overflows u16 memory register.");
		},
		ParseError::NotASCII => {
			write_error(line, line_num, ins_ptr, "Found unicode character! Unicode not supported; ASCII only.");
		},
		ParseError::CInsNop => {
			write_error(line, line_num, ins_ptr, "Invalid c-instruction; has no effect! Requires a Dest or Jump term.");
		},
	}
}

fn write_ram_exhausted_error() {
	println!("RAM exhausted! Assembly terminated!");
}

fn write_rom_exhausted_error(line: &str, line_num: u32, ins_ptr: u16) {
	write_error(line, line_num, ins_ptr, "ROM exhausted! Assembly terminated!");
}

pub fn assemble<R: ?Sized, W: ?Sized>(asm_in: &mut R, bin_out: &mut W) -> io::Result<(u32, u16)>
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
	for line_result in asm_in.lines() {
		line_count += 1;
		let line = line_result?;
		match parse_ins(&line, ins_ptr, &mut sym_key_table, &mut sym_val_table){
			Ok(Some(ins @ Ins::L1{..})) => {
				inss.push(ins);
			},
			Ok(Some(ins)) => {
				inss.push(ins);
				ins_ptr += 1;
			},
			Ok(None) => {
				continue; // skip comment and whitespace lines
			},
			Err(e) => {
				write_parse_error(&e, &line, line_count, ins_ptr);
				error_count += 1;
				ins_ptr += 1;
				if error_count >= MAX_PARSE_ERRORS {
					return Ok((line_count, ins_ptr));
				}
			},
		}
		if ins_ptr >= MAX_ROM_ADDRESS {
			write_rom_exhausted_error(&line, line_count, ins_ptr);
			bin_out.flush()?;
			return Ok((line_count, ins_ptr));
		}
	}

	// Distribute RAM addresses to variables...

	for (ram_address, usage) in &mut sym_val_table {
		if *usage == SymUse::ARAM && *ram_address == DEFAULT_RAM_ADDRESS {
			*ram_address = next_var_ram_address;
			next_var_ram_address += 1;
		}
		if next_var_ram_address >= SCR_RAM_ADDRESS {
			write_ram_exhausted_error();
			bin_out.flush()?;
			return Ok((line_count, ins_ptr));
		}
	}

	// Encode instructions and write to disk...

	for ins in inss {
		if let Some(bin_ins) = encode_ins(&ins, &sym_val_table) {
			writeln!(bin_out, "{:016b}", bin_ins)?;
		}
	}

	bin_out.flush()?;
	Ok((line_count, ins_ptr))
}

#[cfg(test)]
mod tests {
	use std::io::{BufReader, Cursor, BufWriter};
	use std::collections::HashSet;
	use std::fs;
	use super::*;

	fn get_programs_to_test() -> std::io::Result<HashSet<String>> {
		let mut programs = HashSet::new();
		for entry in fs::read_dir("test")? {
			let path = entry?.path();
			if fs::metadata(&path)?.is_file() {
				if let Some(filename) = path.file_stem() {
					programs.insert(filename.to_string_lossy().to_string());
				}
			}
		}
		Ok(programs)
	}

	fn test_assemble_program(asm_file: &str, bin_file: &str){
		use std::fs::File;

		let asm_pong = File::open(asm_file).unwrap();
		let mut asm_in = BufReader::new(asm_pong);

		let bin_pong = File::open(bin_file).unwrap();
		let expected_bin_code = BufReader::new(bin_pong);

		let mut actual_bin_code = BufWriter::new(Cursor::new(Vec::new()));
		assemble(&mut asm_in, &mut actual_bin_code).unwrap();

		let expected_iter = expected_bin_code.lines();
		let actual_iter = actual_bin_code.get_ref().get_ref().lines();

		for (ins_num, (expected, actual)) in expected_iter.zip(actual_iter).enumerate() {
			assert_eq!((ins_num, expected.unwrap()), (ins_num, actual.unwrap()));
		}
	}

	#[test]
	fn test_assemble_progams(){
		for program in get_programs_to_test().unwrap() {
			let asm_file = format!("test/{}.asm", program);
			let bin_file = format!("test/{}.hack", program);
			println!("Test assembling program: {} -> {}", &asm_file, &bin_file);
			test_assemble_program(&asm_file, &bin_file);
		}
	}
}
