#![allow(dead_code)]

mod char_reader;
mod errors;

use compact_str::CompactString;
use lazy_static::lazy_static;
use std::io::{self, BufRead};
use std::collections::HashSet;
use std::str::FromStr;
use regex::Regex;

use char_reader::CharReader;
use errors::*;

lazy_static! {
	static ref SYMBOL_SET: HashSet<char> = {
		HashSet::from(['{', '}', '(', ')', '[', ']', '.', ',', ';', '+', 
			'-', '*', '/', '&', '|', '<', '>', '=', '~'
		])
	};
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Keyword {
	Class, Method, Function, Constructor, Int, Boolean, Char, Void, True, False,
	Null, This, Let, Do, If, Else, While, Return, Field, Static, Var,
}

impl FromStr for Keyword {
	type Err = ();
	fn from_str(word: &str) -> Result<Self, Self::Err> {
		return match word {
			"class"       => Ok(Keyword::Class),
			"constructor" => Ok(Keyword::Constructor),
			"function"    => Ok(Keyword::Function),
			"method"      => Ok(Keyword::Method),
			"field"       => Ok(Keyword::Field),
			"static"      => Ok(Keyword::Static),
			"var"         => Ok(Keyword::Var),
			"int"         => Ok(Keyword::Int),
			"char"        => Ok(Keyword::Char),
			"boolean"     => Ok(Keyword::Boolean),
			"void"        => Ok(Keyword::Void),
			"true"        => Ok(Keyword::True),
			"false"       => Ok(Keyword::False),
			"null"        => Ok(Keyword::Null),
			"this"        => Ok(Keyword::This),
			"let"         => Ok(Keyword::Let),
			"do"          => Ok(Keyword::Do),
			"if"          => Ok(Keyword::If),
			"else"        => Ok(Keyword::Else),
			"while"       => Ok(Keyword::While),
			"return"      => Ok(Keyword::Return),
			_             => Err(())
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	Keyword(Keyword),
	Symbol(char),
	IntConst(u16),
	StrConst(CompactString),
	Identifier(CompactString),
}

pub struct Tokenizer<R: BufRead> {
	chars: CharReader<R>,
	buffer: String,
	token: Option<Token>,
}

impl<R: BufRead> Tokenizer<R> {
	pub fn new(chars: CharReader<R>) -> Self {
		Tokenizer{chars, buffer: String::new(), token: None}
	}

	pub fn next(&mut self) -> Result<Option<Token>, TokenError> {
		if self.token.is_none() {
			self.read_token()
		}
		else {
			let token = self.token.clone();
			self.token = None;
			Ok(token)
		}
	}

	pub fn peek(&mut self) -> Result<Option<Token>, TokenError> {
		if self.token.is_none() {
			self.token = self.read_token()?;
		}
		Ok(self.token.clone())
	}

	pub fn get_line(&self) -> &str {
		self.chars.get_line()
	}

	pub fn get_line_num(&self) -> usize {
		self.chars.get_line_num()
	}

	fn read_token(&mut self) -> Result<Option<Token>, TokenError> {
		self.buffer.clear();
		while let Some(c) = self.chars.next()? {
			if c.is_whitespace() {
				continue;
			}
			else if c == '/' {
				if let Some(c_next) = self.chars.peek()? {
					if c_next == '*' {
						self.skip_multi_line_comment()?;
					}
					else if c_next == '/' {
						self.skip_line_comment()?;
					}
					else {
						return Ok(Some(Token::Symbol('/')));
					}
				}
			}
			else if c == '"' {
				self.read_string_const()?;
				return Ok(Some(Token::StrConst(CompactString::from(self.buffer.as_str()))));
			}
			else if SYMBOL_SET.contains(&c) {
				return Ok(Some(Token::Symbol(c)));
			}
			else {
				self.buffer.push(c);
				break;
			}
		}
		if self.buffer.is_empty() {
			return Ok(None);
		}
		while let Some(c) = self.chars.peek()? {
			if c.is_whitespace() || SYMBOL_SET.contains(&c) {
				break;
			}
			else {
				self.buffer.push(c);
				self.chars.next()?;
			}
		}
		if let Ok(x) = self.buffer.parse::<u16>(){
			return Ok(Some(Token::IntConst(x)));
		}
		if let Ok(keyword) = self.buffer.parse::<Keyword>() {
			return Ok(Some(Token::Keyword(keyword)));
		}
		lazy_static! {
			static ref RX_IDENTIFIER: Regex = Regex::new(r"^[a-zA-Z_]+[a-zA-Z_\d]*").expect("RX_IDENTIFIER invalid!");
		}
		if RX_IDENTIFIER.is_match(&self.buffer) {
			return Ok(Some(Token::Identifier(CompactString::from(self.buffer.as_str()))));
		}
		Err(TokenError::InvalidToken(CompactString::from(self.buffer.as_str())))
	}

	fn skip_line_comment(&mut self) -> Result<(), io::Error> {
		self.chars.next()?;
		while let Some(c) = self.chars.next()? {
			if c == '\n' { break }
		}
		Ok(())
	}

	fn skip_multi_line_comment(&mut self) -> Result<(), io::Error> {
		self.chars.next()?;
		while let Some(c) = self.chars.next()? {
			if let Some(c_next) = self.chars.next()? {
				if c == '*' && c_next == '/' { break }
			}
		}
		Ok(())
	}

	fn read_string_const(&mut self) -> Result<(), io::Error> {
		while let Some(c) = self.chars.next()? {
			if c == '"' { break; }
			self.buffer.push(c);
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::io::{BufReader, Cursor};
	use super::*;

	#[test]
	fn test_tokenizer(){
		let jack_code = r#"
			// This file is part of www.nand2tetris.org
			// and the book "The Elements of Computing Systems"
			// by Nisan and Schocken, MIT Press.
			// File name: projects/10/Square/Main.jack
			
			// (derived from projects/09/Square/Main.jack, with testing additions)
			
			class Main {
			    static boolean test;    // Added for testing -- there is no static Keyword
			                            // in the Square files.
			    function void main() {
			      var SquareGame game;
			      let game = SquareGame.new();
			      do game.run();
			      do game.dispose();
			      return;
			    }
			
			    function void more() {  // Added to test Jack syntax that is not used in
			        var int i, j;       // the Square files.
			        var String s;
			        var Array a;
			        if (false) {
			            let s = "string constant";
			            let s = null;
			            let a[1] = a[2];
			        }
			        else {              // There is no else Keyword in the Square files.
			            let i = i * (-j);
			            let j = j / (-2);   // note: unary negate constant 2
			            let i = i | j;
			        }
			        return;
			    }
			}
		"#.to_string();

		let expected = [
			Token::Keyword(Keyword::Class),
			Token::Identifier(CompactString::from("Main")),
			Token::Symbol('{'),
			Token::Keyword(Keyword::Static),
			Token::Keyword(Keyword::Boolean),
			Token::Identifier(CompactString::from("test")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Function),
			Token::Keyword(Keyword::Void),
			Token::Identifier(CompactString::from("main")),
			Token::Symbol('('),
			Token::Symbol(')'),
			Token::Symbol('{'),
			Token::Keyword(Keyword::Var),
			Token::Identifier(CompactString::from("SquareGame")),
			Token::Identifier(CompactString::from("game")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("game")),
			Token::Symbol('='),
			Token::Identifier(CompactString::from("SquareGame")),
			Token::Symbol('.'),
			Token::Identifier(CompactString::from("new")),
			Token::Symbol('('),
			Token::Symbol(')'),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Do),
			Token::Identifier(CompactString::from("game")),
			Token::Symbol('.'),
			Token::Identifier(CompactString::from("run")),
			Token::Symbol('('),
			Token::Symbol(')'),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Do),
			Token::Identifier(CompactString::from("game")),
			Token::Symbol('.'),
			Token::Identifier(CompactString::from("dispose")),
			Token::Symbol('('),
			Token::Symbol(')'),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Return),
			Token::Symbol(';'),
			Token::Symbol('}'),
			Token::Keyword(Keyword::Function),
			Token::Keyword(Keyword::Void),
			Token::Identifier(CompactString::from("more")),
			Token::Symbol('('),
			Token::Symbol(')'),
			Token::Symbol('{'),
			Token::Keyword(Keyword::Var),
			Token::Keyword(Keyword::Int),
			Token::Identifier(CompactString::from("i")),
			Token::Symbol(','),
			Token::Identifier(CompactString::from("j")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Var),
			Token::Identifier(CompactString::from("String")),
			Token::Identifier(CompactString::from("s")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Var),
			Token::Identifier(CompactString::from("Array")),
			Token::Identifier(CompactString::from("a")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::If),
			Token::Symbol('('),
			Token::Keyword(Keyword::False),
			Token::Symbol(')'),
			Token::Symbol('{'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("s")),
			Token::Symbol('='),
			Token::StrConst(CompactString::from("string constant")),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("s")),
			Token::Symbol('='),
			Token::Keyword(Keyword::Null),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("a")),
			Token::Symbol('['),
			Token::IntConst(1),
			Token::Symbol(']'),
			Token::Symbol('='),
			Token::Identifier(CompactString::from("a")),
			Token::Symbol('['),
			Token::IntConst(2),
			Token::Symbol(']'),
			Token::Symbol(';'),
			Token::Symbol('}'),
			Token::Keyword(Keyword::Else),
			Token::Symbol('{'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("i")),
			Token::Symbol('='),
			Token::Identifier(CompactString::from("i")),
			Token::Symbol('*'),
			Token::Symbol('('),
			Token::Symbol('-'),
			Token::Identifier(CompactString::from("j")),
			Token::Symbol(')'),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("j")),
			Token::Symbol('='),
			Token::Identifier(CompactString::from("j")),
			Token::Symbol('/'),
			Token::Symbol('('),
			Token::Symbol('-'),
			Token::IntConst(2),
			Token::Symbol(')'),
			Token::Symbol(';'),
			Token::Keyword(Keyword::Let),
			Token::Identifier(CompactString::from("i")),
			Token::Symbol('='),
			Token::Identifier(CompactString::from("i")),
			Token::Symbol('|'),
			Token::Identifier(CompactString::from("j")),
			Token::Symbol(';'),
			Token::Symbol('}'),
			Token::Keyword(Keyword::Return),
			Token::Symbol(';'),
			Token::Symbol('}'),
			Token::Symbol('}'),
		];

		let reader = BufReader::new(Cursor::new(jack_code.into_bytes()));
		let chars = CharReader::new(reader);

		let mut tokens = Tokenizer::new(chars);
		let mut expect = expected.into_iter();

		while let Ok(Some(token)) = tokens.peek() {
			let ex = expect.next().unwrap();
			assert_eq!(ex, token);
			assert_eq!(ex, tokens.next().unwrap().unwrap());
		}
	}
}
