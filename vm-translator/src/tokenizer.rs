use compact_str::CompactString;
use std::io::{self, BufRead};
use std::str::FromStr;
use regex::Regex;

pub enum VmCmd {
	Function,
	Label,
	IfGoto,
	Goto,
	Call,
	Add,
	Sub,
	Neg,
	And,
	Or,
	Not,
	Eq,
	Lt,
	Gt,
}

pub enum VmSegment {
	Argument,
	Local,
	Static,
	Constant,
	This,
	That,
	Pointer,
	Temp,
}

pub enum VmToken {
	Command(VmCmd),
	Segment(VmSegment),
	Identifier(CompactString),
	IntConstant(u16),
}

impl FromStr for VmToken {
	type Err = ParseError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(VmToken::IntConstant(0))
	}
}

pub enum ParseError {
}

pub enum TokenError {
	ParseError(ParseError),
	IoError(io::Error),
}

impl From<io::Error> for TokenError {
	fn from(error: io::Error) -> Self {
		TokenError::IoError(error)
	}
}

impl From<ParseError> for TokenError {
	fn from(error: ParseError) -> Self {
		TokenError::ParseError(error)
	}
}

pub struct Tokenizer<R: BufRead> {
	reader: R,
	line: String,
	tokens: Vec<VmToken>,
	regex: Regex,
}

impl<R: BufRead> Tokenizer<R> {
	pub fn new(reader: R) -> Self {
		Tokenizer{
			reader, 
			line: String::new(), 
			tokens: Vec::new(),
			regex: Regex::new(r"[\w.$:]+").unwrap(),
		}
	}
}

impl<R: BufRead> Iterator for Tokenizer<R> {
	type Item = Result<VmToken, TokenError>;
	fn next(&mut self) -> Option<Self::Item> {
		if self.tokens.is_empty() {
			loop {
				self.line.clear();
				match self.reader.read_line(&mut self.line) {
					Ok(0) => return None,
					Ok(n) => n,
					Err(e) => return Some(Err(TokenError::from(e))),
				};
				let mut s = self.line.as_mut_str();
				if let Some(pos) = s.find("//"){
					let (code, _comment) = s.split_at_mut(pos);
					s = code;
				}
				for token in self.regex.find_iter(s) {
					println!("{}", token.as_str());
					match token.as_str().parse::<VmToken>() {
						Ok(t) => self.tokens.push(t),
						Err(e) => return Some(Err(TokenError::from(e))),
					}
				}
				if !self.tokens.is_empty() {
					break;
				}
			}
			self.tokens.reverse();
		}
		match self.tokens.pop() {
			Some(t) => Some(Ok(t)),
			None => None,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn test(){
		use std::io::{BufReader, Cursor};
		use super::*;

		let vm_code = "\
			// This file is part of www.nand2tetris.org
			// and the book \"The Elements of Computing Systems\"
			// by Nisan and Schocken, MIT Press.
			// File name: projects/08/FunctionCalls/SimpleFunction/SimpleFunction.vm
			
			// Performs a simple calculation and returns the result.
			function SimpleFunction.test 2
			push local 0
			push local 1 // another comment
			add
			not//comment   
			push argument 0//comment
			add
			push argument 1
			sub
			return".to_string();

		let reader = BufReader::new(Cursor::new(vm_code.into_bytes()));
		let tokenizer = Tokenizer::new(reader);

		for _ in tokenizer {
			assert_eq!(0,0);
		}
	}
}
