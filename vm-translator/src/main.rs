use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::fs::File;
use crate::coder::*;
use crate::tokenizer::*;
use crate::parser::*;
use crate::errors::*;

mod errors;
mod tokenizer;
mod parser;
mod coder;
mod cli;

fn translate_file<W: Write>(file: PathBuf, coder: &mut Coder, ctx: &mut TranslationContext, out_file: &mut W) -> Result<(), TranslationError> {
	let vm_file = BufReader::new(File::open(file)?);
	let tokenizer = Tokenizer::new(vm_file);
	let mut parser = Parser::new(tokenizer);
	while let Some(ins) = parser.next() {
		ctx.line.clear();
		ctx.line.insert_str(0, parser.get_line());
		ctx.line_num = parser.get_line_num();
		let ins = ins?;
		if let VmIns::Function{ref name, ..} = ins {
			ctx.ins_ctx.vm_function_name = name.clone();
		}
		coder.write_vm_ins(out_file, ins, &ctx.ins_ctx)?;
	}
	Ok(())
}

fn translate<W: Write>(in_files: Vec<PathBuf>, out_file: &mut W, ctx: &mut TranslationContext) -> Result<(), TranslationError> {
	let mut coder = Coder::new();
	coder.write_core_impl(out_file)?;
	for path in in_files {
		ctx.filepath = path.clone();
		ctx.ins_ctx.vm_file_name = path.file_stem().unwrap().to_string_lossy().to_string().into();
		translate_file(path, &mut coder, ctx, out_file)?;
	}
	Ok(())
}

fn main() {
	let args = cli::parse_args();
	let out_file = match File::create(args.output) {
		Ok(file) => file,
		Err(e) => {
			println!("error: failed to create output .asm file: {}", e);
			std::process::exit(0);
		}
	};
	let mut buf_out_file = BufWriter::new(out_file);
	let mut ctx = TranslationContext::new();
	match translate(args.input, &mut buf_out_file, &mut ctx) {
		Ok(()) => (),
		Err(e) => write_translation_error(e, &mut ctx),
	}
}
