mod tokenizer;
mod parser;
mod coder;
mod cli;

fn main() {
	let args = cli::parse_args();
	println!("{:?}", args);
}
