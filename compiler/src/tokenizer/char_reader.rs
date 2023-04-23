use std::io::{self, BufRead};
use normalize_line_endings::normalized;

pub struct CharReader<R: BufRead> {
	reader: R,
	full_line: String,
	read_line: String,
	temp_line: String,
	line_num: usize,
	char_offset: usize,
}

impl<R: BufRead> CharReader<R> {
	pub fn new(reader: R) -> Self {
		CharReader{
			reader,
			full_line: String::new(),
			temp_line: String::new(),
			read_line: String::new(),
			line_num: 0,
			char_offset: 0
		}
	}

	fn fill_read_line(&mut self) -> Result<usize, io::Error> {
		self.full_line.clear();
		let n = match self.reader.read_line(&mut self.full_line){
			Ok(0) => return Ok(0),
			Ok(n) => n,
			Err(e) => return Err(e),
		};
		self.temp_line.clear();
		for c in normalized(self.full_line.chars()) {
			self.temp_line.push(c);
		}
		self.read_line.clear();
		for c in self.temp_line.chars().rev() {
			self.read_line.push(c);
		}
		self.line_num += 1;
		self.char_offset = 0;
		Ok(n)
	}

	pub fn next(&mut self) -> Result<Option<char>, io::Error> {
		if self.read_line.is_empty() {
			self.fill_read_line()?;
		}
		if let Some(c) = self.read_line.pop() {
			self.char_offset += 1;
			return Ok(Some(c));
		}
		Ok(None)
	}

	pub fn peek(&mut self) -> Result<Option<char>, io::Error> {
		if self.read_line.is_empty() {
			self.fill_read_line()?;
		}
		if !self.read_line.is_empty() {
			return Ok(Some(self.read_line.chars().rev().next().unwrap()));
		}
		Ok(None)
	}

	pub fn get_line(&self) -> &str {
		self.full_line.as_str()
	}

	pub fn get_line_num(&self) -> usize {
		self.line_num
	}

	pub fn get_char_offset(&self) -> usize {
		self.char_offset
	}
}

#[cfg(test)]
mod tests {
	use std::io::{BufReader, Cursor};
	use super::*;

	#[test]
	fn test_char_reader(){
		let data = "ab\n\r\nde\rop\r\n\r\n\nadw".to_string();
		let reader = BufReader::new(Cursor::new(data.into_bytes()));
		let mut char_reader = CharReader::new(reader);
		let expected = ['a','b','\n','\n','d','e','\n','o','p','\n','\n','\n','a','d','w'].to_vec();
		for i in 0..expected.len() {
			let c = expected[i];
			assert_eq!(c, char_reader.peek().unwrap().unwrap());
			assert_eq!(c, char_reader.next().unwrap().unwrap());
		}
	}
}
