use compact_str::CompactString;
use core::ops::Range;
use std::path::PathBuf;
use std::io;
use crate::tokenizer::{VmToken, VmSeg};
use crate::InsContext;

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

pub enum CodeError {
	IndexOutOfBounds{segment: VmSeg, index: u16, bounds: Range<usize>},
	IoError(io::Error),
}

impl From<io::Error> for CodeError {
	fn from(e: io::Error) -> Self {
		CodeError::IoError(e)
	}
}

pub struct TranslationContext {
	pub filepath: PathBuf,
	pub ins_ctx: InsContext,
	pub line: String,
	pub line_num: usize,
}

impl TranslationContext {
	pub fn new() -> Self {
		TranslationContext{filepath: PathBuf::new(), ins_ctx: InsContext::new(), line: String::new(), line_num: 0}
	}
}

pub enum TranslationError {
	ParseError(ParseError),
	CodeError(CodeError),
	IoError(io::Error),
}

impl From<ParseError> for TranslationError {
	fn from(e: ParseError) -> Self {
		TranslationError::ParseError(e)
	}
}

impl From<CodeError> for TranslationError {
	fn from(e: CodeError) -> Self {
		TranslationError::CodeError(e)
	}
}

impl From<io::Error> for TranslationError {
	fn from(e: io::Error) -> Self {
		TranslationError::IoError(e)
	}
}

fn write_error(msg: &str, ctx: &TranslationContext) {
	println!("{}, on line:\n[{}] {}", msg, ctx.line_num, ctx.line);
}

fn write_io_error(e: io::Error){
	println!("io error: {}", e);
}

fn write_token_error(e: TokenError, ctx: &TranslationContext){
	match e {
		TokenError::IoError(e) => write_io_error(e),
		TokenError::InvalidToken{word} => {
			write_error(format!("token error: invalid token '{}'", word).as_str(), ctx);
		},
	}
}

fn write_parse_error(e: ParseError, ctx: &TranslationContext){
	match e {
		ParseError::ExpectedCommand{received} => {
			write_error(format!("parse error: expected command, received {}", received.unwrap()).as_str(), ctx);
		},
		ParseError::ExpectedIdentifier{received} => {
			write_error(format!("parse error: expected identifier, received {}", received.unwrap()).as_str(), ctx);
		},
		ParseError::ExpectedIntConst{received} => {
			write_error(format!("parse error: expected integer constant, received {}", received.unwrap()).as_str(), ctx);
		},
		ParseError::ExpectedSegment{received} => {
			write_error(format!("parse error: expected segment, received {}", received.unwrap()).as_str(), ctx);
		},
		ParseError::TokenError(e) => {
			write_token_error(e, ctx);
		},
	}
}

fn write_code_error(e: CodeError, ctx: &TranslationContext){
	match e {
		CodeError::IoError(e) => write_io_error(e),
		CodeError::IndexOutOfBounds{segment, index, bounds} => {
			let msg = format!("code error: index '{}' overflows segment '{}'; segment bounds '[{},{}]'", 
				index, segment, bounds.start, bounds.end);
			write_error(&msg, ctx);
		},
	}
}

pub fn write_translation_error(e: TranslationError, ctx: &TranslationContext) {
	match e {
		TranslationError::IoError(e) => write_io_error(e),
		TranslationError::ParseError(e) => write_parse_error(e, ctx),
		TranslationError::CodeError(e) => write_code_error(e, ctx),
	}
}

