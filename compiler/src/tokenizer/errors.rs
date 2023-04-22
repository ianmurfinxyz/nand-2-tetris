use compact_str::CompactString;
use std::io;

#[derive(Debug)]
pub enum TokenError {
	InvalidToken(CompactString),
	IoError(io::Error),
}

impl From<io::Error> for TokenError {
	fn from(e: io::Error) -> Self {
		TokenError::IoError(e)
	}
}
