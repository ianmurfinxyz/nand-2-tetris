use std::io::BufRead;
use compact_str::CompactString;
use crate::tokenizer::*;

#[derive(Debug, PartialEq)]
pub enum VmIns {
	Function{name: CompactString, locals_count: u16},
	Call{function: CompactString, args_count: u16},
	Push{segment: VmSeg, index: u16},
	Pop{segment: VmSeg, index: u16},
	Label{label: CompactString},
	IfGoto{label: CompactString},
	Goto{label: CompactString},
	Return,
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

#[derive(Debug)]
pub enum ParseError {
	ExpectedCommand{received: Option<VmToken>},
	ExpectedIdentifier{received: Option<VmToken>},
	ExpectedIntConst{received: Option<VmToken>},
	ExpectedSegment{received: Option<VmToken>},
	TokenError(TokenError),
}

impl From<TokenError> for ParseError {
	fn from(e: TokenError) -> Self {
		ParseError::TokenError(e)
	}
}

pub struct Parser<R: BufRead> {
	tokenizer: Tokenizer<R>,
}

impl<R: BufRead> Parser<R> {
	fn new(tokenizer: Tokenizer<R>) -> Self {
		Parser{tokenizer}
	}

	fn parse_identifier(&mut self) -> Result<CompactString, ParseError> {
		return match self.tokenizer.next() {
			Some(Ok(VmToken::Identifier(identifier))) => Ok(identifier),
			Some(Err(e)) => Err(ParseError::from(e)),
			Some(Ok(token)) => Err(ParseError::ExpectedIdentifier{received: Some(token)}),
			None => Err(ParseError::ExpectedIdentifier{received: None}),
		}
	}

	fn parse_int_const(&mut self) -> Result<u16, ParseError> {
		return match self.tokenizer.next() {
			Some(Ok(VmToken::IntConst(int))) => Ok(int),
			Some(Err(e)) => Err(ParseError::from(e)),
			Some(Ok(token)) => Err(ParseError::ExpectedIntConst{received: Some(token)}),
			None => Err(ParseError::ExpectedIntConst{received: None}),
		}
	}

	fn parse_segment(&mut self) -> Result<VmSeg, ParseError> {
		return match self.tokenizer.next() {
			Some(Ok(VmToken::Segment(segment))) => Ok(segment),
			Some(Err(e)) => Err(ParseError::from(e)),
			Some(Ok(token)) => Err(ParseError::ExpectedSegment{received: Some(token)}),
			None => Err(ParseError::ExpectedSegment{received: None}),
		}
	}

	fn parse_command(&mut self, cmd: VmCmd) -> Result<VmIns, ParseError> {
		match cmd {
			VmCmd::Function => Ok(VmIns::Function{name: self.parse_identifier()?, locals_count: self.parse_int_const()?}),
			VmCmd::Return => Ok(VmIns::Return),
			VmCmd::Label => Ok(VmIns::Label{label: self.parse_identifier()?}),
			VmCmd::IfGoto => Ok(VmIns::IfGoto{label: self.parse_identifier()?}),
			VmCmd::Goto => Ok(VmIns::Goto{label: self.parse_identifier()?}),
			VmCmd::Call => Ok(VmIns::Call{function: self.parse_identifier()?, args_count: self.parse_int_const()?}),
			VmCmd::Push => Ok(VmIns::Push{segment: self.parse_segment()?, index: self.parse_int_const()?}),
			VmCmd::Pop => Ok(VmIns::Pop{segment: self.parse_segment()?, index: self.parse_int_const()?}),
			VmCmd::Add => Ok(VmIns::Add),
			VmCmd::Sub => Ok(VmIns::Sub),
			VmCmd::Neg => Ok(VmIns::Neg),
			VmCmd::And => Ok(VmIns::And),
			VmCmd::Or => Ok(VmIns::Or),
			VmCmd::Not => Ok(VmIns::Not),
			VmCmd::Eq => Ok(VmIns::Eq),
			VmCmd::Lt => Ok(VmIns::Lt),
			VmCmd::Gt => Ok(VmIns::Gt),
		}
	}
}

impl<R: BufRead> Iterator for Parser<R> {
	type Item = Result<VmIns, ParseError>;
	fn next(&mut self) -> Option<Self::Item> {
		return match self.tokenizer.next() {
			Some(Ok(VmToken::Command(cmd))) => Some(self.parse_command(cmd)),
			Some(Ok(token)) => Some(Err(ParseError::ExpectedCommand{received: Some(token)})),
			Some(Err(e)) => Some(Err(ParseError::from(e))),
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
		let tokenizer = Tokenizer::new(reader);
		let mut parser = Parser::new(tokenizer);

		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Function{name: CompactString::from("SimpleFunction.test"), locals_count: 2});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Local, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Local, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Add);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Not);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Add);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Sub);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Return);
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
		let tokenizer = Tokenizer::new(reader);
		let mut parser = Parser::new(tokenizer);

		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Function{name: CompactString::from("Sys.init"), locals_count: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 6});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 8});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Call{function: CompactString::from("Class1.set"), args_count: 2});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Temp, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 23});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 15});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Call{function: CompactString::from("Class2.set"), args_count: 2});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Temp, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Call{function: CompactString::from("Class1.get"), args_count: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Call{function: CompactString::from("Class2.get"), args_count: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Label{label: CompactString::from("WHILE")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Goto{label: CompactString::from("WHILE")});
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
		let tokenizer = Tokenizer::new(reader);
		let mut parser = Parser::new(tokenizer);

		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Pointer, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::That, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::That, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 2});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Sub);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Label{label: CompactString::from("MAIN_LOOP_START")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::IfGoto{label: CompactString::from("COMPUTE_ELEMENT")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Goto{label: CompactString::from("END_PROGRAM")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Label{label: CompactString::from("COMPUTE_ELEMENT")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::That, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::That, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Add);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::That, index: 2});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Pointer, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Add);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Pointer, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Push{segment: VmSeg::Constant, index: 1});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Sub);
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Pop{segment: VmSeg::Argument, index: 0});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Goto{label: CompactString::from("MAIN_LOOP_START")});
		assert_eq!(parser.next().unwrap().unwrap(), VmIns::Label{label: CompactString::from("END_PROGRAM")});
	}
}
