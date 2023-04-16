use clap::Parser;
use std::path::PathBuf;
use std::fs;

const ABOUT_HELP: &'static str = "\
Translate intermediate Hack platform VM code to assembly. Input is a set of 
vm code files; translation links all input files into a single assembly.";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = ABOUT_HELP)]
struct ClapArgs {
	#[arg(name = "input", help = "code to translate; file/s and/or directory/s")]
	input: Vec<PathBuf>,
	#[arg(name = "output", short, long, help = "path to output assembly", default_value = "out.asm")]
	output: String,
}

#[derive(Debug)]
pub struct CliArgs {
	input: Vec<PathBuf>,
	output: String,
}

enum InputError {
	NotFileOrDir(PathBuf),
	IoError(std::io::Error),
}

impl From<std::io::Error> for InputError {
	fn from(e: std::io::Error) -> Self {
		InputError::IoError(e)
	}
}

fn gather_files_in_dir(path: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
	let mut files = vec![];
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_file(){
			files.push(path);
		}
		else if path.is_dir() {
			files.extend(gather_files_in_dir(&path)?);
		}
	}
	Ok(files)
}

fn gather_input_files(input: Vec<PathBuf>) -> Result<Vec<PathBuf>, InputError> {
	let mut in_files = vec![];
	for path in input {
		if path.is_file() {
			in_files.push(path);
		}
		else if path.is_dir() {
			in_files.extend(gather_files_in_dir(&path)?);
		}
		else {
			return Err(InputError::NotFileOrDir(path));
		}
	}
	Ok(in_files)
}

pub fn parse_args() -> CliArgs {
	let args = ClapArgs::parse();

	let mut in_files = match gather_input_files(args.input){
		Ok(files) => files,
		Err(InputError::NotFileOrDir(e)) => {
			println!("error: cannot find file or directory at path '{}'", e.to_string_lossy());
			std::process::exit(0);
		},
		Err(InputError::IoError(e)) => {
			println!("error: invalid input! {}", e);
			std::process::exit(0);
		},
	};

	in_files = in_files.into_iter().filter(|f| {
		let ext = f.extension();
		!ext.is_none() && ext.unwrap() == "vm"
	}).collect();

	CliArgs{input: in_files, output: args.output}
}
