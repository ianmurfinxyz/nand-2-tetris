use compact_str::CompactString;
use std::io::{self, BufRead};
use std::string::FromUtf8Error;
use std::mem::swap;
use std::ops::Range;

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
	buf: Vec<u8>,
	str: String,
	pos: usize,
	end: usize,
}

impl<R: BufRead> Tokenizer<R> {
	fn new(reader: R) -> Self {
		Tokenizer{reader, buf: Vec::with_capacity(MAX_READ_SIZE_BYTES), str: String::new(), pos: 0, end: 0}
	}

	fn find_next_token_range(&mut self) -> Option<Range<usize>> {
		let start = {
			let mut it = self.str[self.pos..].char_indices().peekable();
			// Search for the first non-whitespace character that is not part of a comment.
			loop {
				if let Some((i0, c0)) = it.next() {
					if c0.is_whitespace() {
						continue;
					}
					// If we hit a comment...eat it!
					if c0 == '/' && it.peek() == Some(&(i0 + 1, '/')) {
						loop {
							if let Some((_, c1)) = it.next() {
								if let Some((_, c2)) = it.peek() {
									match (c1, c2) {
										('\n', _) => {
											break;
										},
										('\r', '\n') => {
											it.next();
											break;
										},
										('\r', _) => {
											break;
										},
										_ => continue,
									}
								}
							}
						}
						continue;
					}
					break Some(i0)
				}
				else {
					break None
				}
			}
		};

		if start.is_none() {
			return None;
		}

		// Search for the first whitespace character or the start of a comment.
		let end = {
			let mut it = self.str[start.unwrap()..].char_indices().peekable();
			loop {
				if let Some((i0, c0)) = it.next() {
					if c0.is_whitespace() || (c0 == '/' && it.peek() == Some(&(i0 + 1, '/'))) {
						break Some(i0);
					}
				}
				else {
					break None
				}
			}
		};

		if end.is_none() {
			return None;
		}

		Some(start.unwrap()..end.unwrap())
	}

	fn read_more_bytes(&mut self) -> Result<(), TokenError> {
		// The borrow checker will not allow us to uninitialise the buf and str
		// from a &mut self, so to transfer ownership from the buf to the str
		// and visa versa we need to ensure the self.buf and self.str remain
		// initialised, which we can do by swapping in empty versions.
		let mut buf = Vec::new();
		let mut str = String::new();
		swap(&mut self.buf, &mut buf);
		swap(&mut self.str, &mut str);

		// We are trying to transfer ownership between these buffers...make sure
		// we havn't accidentally deallocated our memory.
		debug_assert!(buf.capacity() > 0 || str.capacity() > 0);

		// Transfer ownership of the buffer back to the vec.
		if str.capacity() > 0 {
			debug_assert!(buf.capacity() == 0);
			buf = str.into_bytes();
		}

		debug_assert!(buf.capacity() > 0);

		// Bytes at the end of the buffer we have not yet processed.
		let remaining_bytes_count = self.end - self.pos;

		// If the next token is partially in the buffer, shift it to the start.
		if remaining_bytes_count > 0 {
			let (left, right) = buf.split_at_mut(self.pos);
			left.copy_from_slice(&right);
		}

		self.end = remaining_bytes_count;
		self.pos = 0;

		loop {
			match self.reader.read(&mut buf[self.end..]) {
				Ok(n) => { self.end += n; break; },
				Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
				Err(e) => return Err(TokenError::from(e)),
			};
		}

		// We do self.end += n...safety first!
		debug_assert!(self.end <= MAX_READ_SIZE_BYTES);

		// Will transfer ownership from the buffer to the string.
		str = match String::from_utf8(buf) {
			Ok(s) => s,
			Err(e) => return Err(TokenError::from(e)),
		};

		swap(&mut self.str, &mut str);

		Ok(())
	}
}

impl<R: BufRead> Iterator for Tokenizer<R> {
	type Item = Result<VmToken, TokenError>;

	fn next(&mut self) -> Option<Self::Item> {

		let mut range = self.find_next_token_range();
		if range.is_none() {
			match self.read_more_bytes() {
				Ok(()) => range = self.find_next_token_range(),
				Err(e) => return Some(Err(e)),
			}
		}

		if range.is_none() {
			return None;
		}

		Some(Ok(VmToken::IntConstant(0)))
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

