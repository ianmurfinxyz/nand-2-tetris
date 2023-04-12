use compact_str::CompactString;
use std::io::{self, BufRead};
use std::string::FromUtf8Error;

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

pub enum ParseError {
}

pub enum TokenError {
	ParseError(ParseError),
	Utf8Error(FromUtf8Error),
	IoError(io::Error),
}

impl From<io::Error> for TokenError {
	fn from(error: io::Error) -> Self {
		TokenError::IoError(error)
	}
}

impl From<FromUtf8Error> for TokenError {
	fn from(error: FromUtf8Error) -> Self {
		TokenError::Utf8Error(error)
	}
}

impl From<ParseError> for TokenError {
	fn from(error: ParseError) -> Self {
		TokenError::ParseError(error)
	}
}


// buffered string iteration technique
// https://stackoverflow.com/questions/35385703/read-file-character-by-character-in-rust

const MAX_READ_SIZE_BYTES: usize = 4096;

pub struct Tokenizer<R: BufRead> {
	reader: R,
	buffer: Vec<u8>,
	view: String,
	pos: usize,
	end: usize,
}

impl<R: BufRead> Tokenizer<R> {
	fn new(reader: R) -> Self {
		Tokenizer{reader, buffer: Vec::with_capacity(MAX_READ_SIZE_BYTES), view: String::new(), pos: 0, end: 0}
	}
}

impl<R: BufRead> Iterator for Tokenizer<R> {
	type Item = Result<VmToken, TokenError>;

	fn next(&mut self) -> Option<Self::Item> {

		// Test if the entire next token is currently in the buffer.
		let mut next_token_end_pos = None;
		for (i, c) in self.view[self.pos..].chars().enumerate() {
			if c.is_whitespace() {
				next_token_end_pos = Some(i);
				break;
			}
		};

		// If next token is not in the buffer we need to repopulate the buffer.
		if next_token_end_pos.is_none() {

			// Transfer ownership of the buffer back to the vec.
			if !self.view.is_empty() {
				self.buffer = self.view.into_bytes();
				self.view.clear();
			}

			// Bytes at the end of the buffer we have not yet processed.
			let remaining_bytes_count = self.end - self.pos;

			// If the next token is partially in the buffer, shift it to the start.
			if remaining_bytes_count > 0 {
				self.buffer[0..].copy_from_slice(&self.buffer[self.pos..]);
				self.end = remaining_bytes_count;
				self.pos == 0;
			}

			loop {
				match self.reader.read(&mut self.buffer[self.end..]) {
					Ok(n) => { self.end += n; break; },
					Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
					Err(e) => return Some(Err(TokenError::from(e))),
				};
			}

			// We do self.end += n...safety first!
			debug_assert!(self.end <= MAX_READ_SIZE_BYTES);

			// If we hit EOF...
			if self.end == 0 {
				return None
			}

			// Will transfer ownership from the buffer to the string.
			self.view = match String::from_utf8(self.buffer) {
				Ok(s) => s,
				Err(e) => return Some(Err(TokenError::from(e))),
			};
		}





		None
	}
}



//struct Utf8CharsIter<I: Iterator<Item = io::Result<u8>>> {
//	byte_stream: I,
//	bytes: [u8; 4],
//	len: u8,
//}
//
//impl<I: Iterator<Item = io::Result<u8>>> Utf8CharsIter<I> {
//	fn new(byte_stream: I) -> Self {
//		Utf8CharsIter{byte_stream, bytes: [0,0,0,0], len: 0}
//	}
//}
//
//impl<I: Iterator<Item = io::Result<u8>>> Iterator for Utf8CharsIter<I> {
//	type Item = io::Result<char>;
//	fn next(&mut self) -> Option<Self::Item> {
//		loop {
//			match std::str::from_utf8(&self.bytes[..self.len]) {
//				Ok(s) => {
//					if let Some(c) = s.chars().next(){
//
//					}
//				},
//				Err(e) => {
//				}
//			}
//		}
//	}
//}

