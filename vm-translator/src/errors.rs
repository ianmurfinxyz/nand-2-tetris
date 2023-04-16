use compact_str::CompactString;
use core::ops::Range;
use std::io;
use crate::tokenizer::{VmToken, VmSeg};

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

pub fn log_translation_error(_error: TranslationError) {
	println!("error");
}
