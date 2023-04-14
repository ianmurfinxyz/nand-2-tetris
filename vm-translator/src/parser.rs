use std::io::BufRead;
use compact_str::CompactString;
use crate::tokenizer::*;

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
