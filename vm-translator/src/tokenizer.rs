use std::io::{self, BufRead};
use std::str::FromStr;
use compact_str::CompactString;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VmCmd {
	Function,
	Return,
	Label,
	IfGoto,
	Goto,
	Call,
	Push,
	Pop,
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

impl fmt::Display for VmCmd {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			VmCmd::Function => "function",
			VmCmd::Return   => "return",
			VmCmd::Label    => "label",
			VmCmd::IfGoto   => "if-goto",
			VmCmd::Goto     => "goto",
			VmCmd::Call     => "call",
			VmCmd::Push     => "push",
			VmCmd::Pop     => "pop",
			VmCmd::Add      => "add",
			VmCmd::Sub      => "sub",
			VmCmd::Neg      => "neg",
			VmCmd::And      => "adn",
			VmCmd::Or       => "or",
			VmCmd::Not      => "not",
			VmCmd::Eq       => "eq",
			VmCmd::Lt       => "lt",
			VmCmd::Gt       => "gt",
		};
		write!(f, "{}", s)
	}
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VmSeg {
	Argument,
	Local,
	Static,
	Constant,
	This,
	That,
	Pointer,
	Temp,
}

impl fmt::Display for VmSeg {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let s = match self {
			VmSeg::Argument => "argument",
			VmSeg::Local    => "local",
			VmSeg::Static   => "static",
			VmSeg::Constant => "constant",
			VmSeg::This     => "this",
			VmSeg::That     => "that",
			VmSeg::Pointer  => "pointer",
			VmSeg::Temp     => "temp",
		};
		write!(f, "{}", s)
	}
}

#[derive(Debug, PartialEq)]
pub enum VmToken {
	Command(VmCmd),
	Segment(VmSeg),
	Identifier(CompactString),
	IntConst(u16),
}

impl fmt::Display for VmToken {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl FromStr for VmToken {
	type Err = TokenError;
	fn from_str(word: &str) -> Result<Self, Self::Err> {
		if let Ok(x) = word.parse::<u16>(){
			return Ok(VmToken::IntConst(x));
		}
		let cmd = match word {
			"function" => Some(VmToken::Command(VmCmd::Function)),
			"return"   => Some(VmToken::Command(VmCmd::Return)),
			"label"    => Some(VmToken::Command(VmCmd::Label)),
			"if-goto"  => Some(VmToken::Command(VmCmd::IfGoto)),
			"goto"     => Some(VmToken::Command(VmCmd::Goto)),
			"call"     => Some(VmToken::Command(VmCmd::Call)),
			"push"     => Some(VmToken::Command(VmCmd::Push)),
			"pop"      => Some(VmToken::Command(VmCmd::Pop)),
			"add"      => Some(VmToken::Command(VmCmd::Add)),
			"sub"      => Some(VmToken::Command(VmCmd::Sub)),
			"neg"      => Some(VmToken::Command(VmCmd::Neg)),
			"and"      => Some(VmToken::Command(VmCmd::And)),
			"or"       => Some(VmToken::Command(VmCmd::Or)),
			"not"      => Some(VmToken::Command(VmCmd::Not)),
			"eq"       => Some(VmToken::Command(VmCmd::Eq)),
			"lt"       => Some(VmToken::Command(VmCmd::Lt)),
			"gt"       => Some(VmToken::Command(VmCmd::Gt)),
			_          => None,
		};
		if let Some(t) = cmd {
			return Ok(t);
		}
		let seg = match word {
			"argument" => Some(VmToken::Segment(VmSeg::Argument)),
			"local"    => Some(VmToken::Segment(VmSeg::Local)),
			"static"   => Some(VmToken::Segment(VmSeg::Static)),
			"constant" => Some(VmToken::Segment(VmSeg::Constant)),
			"this"     => Some(VmToken::Segment(VmSeg::This)),
			"that"     => Some(VmToken::Segment(VmSeg::That)),
			"pointer"  => Some(VmToken::Segment(VmSeg::Pointer)),
			"temp"     => Some(VmToken::Segment(VmSeg::Temp)),
			_          => None,
		};
		if let Some(t) = seg {
			return Ok(t);
		}
		lazy_static! {
			static ref TOKEN: Regex = Regex::new(r"[\w.$:]+").expect("error compiling TOKEN regex");
		}
		if TOKEN.is_match(word) {
			return Ok(VmToken::Identifier(CompactString::from(word)));
		}
		Err(TokenError::InvalidToken{word: CompactString::from(word)})
	}
}

#[derive(Debug)]
pub enum TokenError {
	InvalidToken{word: CompactString},
	IoError(io::Error),
}

impl From<io::Error> for TokenError {
	fn from(e: io::Error) -> Self {
		TokenError::IoError(e)
	}
}

pub struct Tokenizer<R: BufRead> {
	reader: R,
	line: String,
	tokens: Vec<VmToken>,
}

impl<R: BufRead> Tokenizer<R> {
	pub fn new(reader: R) -> Self {
		Tokenizer{reader, line: String::new(), tokens: Vec::new()}
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
				lazy_static! {
					static ref WORDS: Regex = Regex::new(r"[\S]+").expect("error compiling WORDS regex");
				}
				for word in WORDS.find_iter(s) {
					match word.as_str().parse::<VmToken>() {
						Ok(t) => self.tokens.push(t),
						Err(e) => return Some(Err(e)),
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
	use std::io::{BufReader, Cursor};
	use super::*;

	// TODO: Implement unit tests for error paths; only tested happy paths!

	#[test]
	fn test_simple_function(){
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
			return
		".to_string();

		let reader = BufReader::new(Cursor::new(vm_code.into_bytes()));
		let mut tokenizer = Tokenizer::new(reader);
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("SimpleFunction.test")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Local));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Local));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Add));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Not));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Add));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Sub));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Return));
		assert_eq!(tokenizer.next().is_none(), true);
	}

	#[test]
	fn test_class_2(){
		let vm_code = "\
			// This file is part of www.nand2tetris.org
			// and the book \"The Elements of Computing Systems\"
			// by Nisan and Schocken, MIT Press.
			// File name: projects/08/FunctionCalls/StaticsTest/Class2.vm
			
			// Stores two supplied arguments in static[0] and static[1].
			function Class2.set 0
			push argument 0
			pop static 0
			push argument 1
			pop static 1
			push constant 0
			return

			// Returns static[0] - static[1].
			function Class2.get 0
			push static 0
			push static 1
			sub
			return
		".to_string();

		let reader = BufReader::new(Cursor::new(vm_code.into_bytes()));
		let mut tokenizer = Tokenizer::new(reader);

		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class2.set")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Static));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Static));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Return));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class2.get")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Static));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Static));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Sub));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Return));
		assert_eq!(tokenizer.next().is_none(), true);
	}

	#[test]
	fn test_sys(){
		let vm_code = "\
			// This file is part of www.nand2tetris.org
			// and the book \"The Elements of Computing Systems\"
			// by Nisan and Schocken, MIT Press.
			// File name: projects/08/FunctionCalls/StaticsTest/Sys.vm
			
			// Tests that different functions, stored in two different 
			// class files, manipulate the static segment correctly. 
			function Sys.init 0
			push constant 6
			push constant 8
			call Class1.set 2
			pop temp 0 // Dumps the return value
			push constant 23
			push constant 15
			call Class2.set 2
			pop temp 0 // Dumps the return value
			call Class1.get 0
			call Class2.get 0
			label WHILE
			goto WHILE
		".to_string();

		let reader = BufReader::new(Cursor::new(vm_code.into_bytes()));
		let mut tokenizer = Tokenizer::new(reader);
		
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Sys.init")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(6));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(8));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Call));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class1.set")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Temp));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(23));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(15));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Call));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class2.set")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Temp));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Call));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class1.get")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Call));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("Class2.get")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Label));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("WHILE")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Goto));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("WHILE")));
		assert_eq!(tokenizer.next().is_none(), true);
	}

	#[test]
	fn test_fibonacci_series(){
		let vm_code = "\
			// This file is part of www.nand2tetris.org
			// and the book \"The Elements of Computing Systems\"
			// by Nisan and Schocken, MIT Press.
			// File name: projects/08/ProgramFlow/FibonacciSeries/FibonacciSeries.vm
			
			// Puts the first argument[0] elements of the Fibonacci series
			// in the memory, starting in the address given in argument[1].
			// Argument[0] and argument[1] are initialized by the test script 
			// before this code starts running.
			
			push argument 1
			pop pointer 1           // that = argument[1]
			
			push constant 0
			pop that 0              // first element in the series = 0
			push constant 1
			pop that 1              // second element in the series = 1
			
			push argument 0
			push constant 2
			sub
			pop argument 0          // num_of_elements -= 2 (first 2 elements are set)
			
			label MAIN_LOOP_START
			
			push argument 0
			if-goto COMPUTE_ELEMENT // if num_of_elements > 0, goto COMPUTE_ELEMENT
			goto END_PROGRAM        // otherwise, goto END_PROGRAM
			
			label COMPUTE_ELEMENT
			
			push that 0
			push that 1
			add
			pop that 2              // that[2] = that[0] + that[1]
			
			push pointer 1
			push constant 1
			add
			pop pointer 1           // that += 1
			
			push argument 0
			push constant 1
			sub
			pop argument 0          // num_of_elements--
			
			goto MAIN_LOOP_START
			
			label END_PROGRAM
		".to_string();

		let reader = BufReader::new(Cursor::new(vm_code.into_bytes()));
		let mut tokenizer = Tokenizer::new(reader);

		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Pointer));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::That));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::That));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Sub));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Label));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("MAIN_LOOP_START")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::IfGoto));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("COMPUTE_ELEMENT")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Goto));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("END_PROGRAM")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Label));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("COMPUTE_ELEMENT")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::That));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::That));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Add));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::That));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Pointer));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Add));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Pointer));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Push));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Constant));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Sub));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Pop));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Segment(VmSeg::Argument));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::IntConst(0));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Goto));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("MAIN_LOOP_START")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Command(VmCmd::Label));
		assert_eq!(tokenizer.next().unwrap().unwrap(), VmToken::Identifier(CompactString::from("END_PROGRAM")));
		assert_eq!(tokenizer.next().is_none(), true);
	}
}
