use std::io::{BufReader, BufWriter};
use std::time::Instant;
use std::fs::File;
use clap::Parser;
use crate::assembler::*;

mod parser;
mod encoder;
mod assembler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Translate a Hack assembly (.asm) file to a Hack binary (.hack) file.")]
struct Args {
		#[arg(name = "asm", help = "path to input assembly .asm file")]
		asm_file_path: String,
		#[arg(name = "out", short, long, help = "path to output binary .hack file", default_value = "out.hack")]
		bin_file_path: String,
}

fn main(){
	let args = Args::parse();

	let asm_file = match File::open(args.asm_file_path) {
		Ok(file) => file,
		Err(e) => {
			println!("error: failed to open input .asm file: {}", e);
			std::process::exit(-1);
		}
	};

	let bin_file = match File::create(args.bin_file_path) {
		Ok(file) => file,
		Err(e) => {
			println!("error: failed to create output .hack file: {}", e);
			std::process::exit(-1);
		}
	};

	let mut asm_reader = BufReader::new(asm_file);
	let mut bin_writer = BufWriter::new(bin_file);

	let now = Instant::now();
	let result = assemble(&mut asm_reader, &mut bin_writer);
	let elapsed = now.elapsed();

	match result {
		Ok((line_count, ins_count)) => {
			println!("Translated {} instructions ({} lines) in {:.2?}", ins_count, line_count, elapsed);
		},
		Err(e) => {
			println!("error: {}", e);
		}
	}
}
