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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Token {
	Keyword(Keyword),
	Symbol(char),
	IntConst(u16),
	StrConst(CompactString),
	Identifier(CompactString),
}

pub struct Tokenizer<R: BufRead> {
	chars: CharReader<R>,
	token: String,
}

impl<R: BufRead> Tokenizer<R> {
	pub fn new(chars: CharReader<R>) -> Self {
		Tokenizer{chars, token: String::new()}
	}

	fn next(&mut self) -> Result<Option<Token>, TokenError> {
		self.token.clear();
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
				return Ok(Some(Token::StrConst(CompactString::from(self.token.as_str()))));
			}
			else if SYMBOL_SET.contains(&c) {
				return Ok(Some(Token::Symbol(c)));
			}
			else {
				self.token.push(c);
				break;
			}
		}
		if self.token.is_empty() {
			return Ok(None);
		}
		while let Some(c) = self.chars.peek()? {
			if c.is_whitespace() || SYMBOL_SET.contains(&c) {
				break;
			}
			else {
				self.token.push(c);
				self.chars.next()?;
			}
		}
		if let Ok(x) = self.token.parse::<u16>(){
			return Ok(Some(Token::IntConst(x)));
		}
		if let Ok(keyword) = self.token.parse::<Keyword>() {
			return Ok(Some(Token::Keyword(keyword)));
		}
		lazy_static! {
			static ref RX_IDENTIFIER: Regex = Regex::new(r"^[a-zA-Z_]+[a-zA-Z_\d]*").expect("RX_IDENTIFIER invalid!");
		}
		if RX_IDENTIFIER.is_match(&self.token) {
			return Ok(Some(Token::Identifier(CompactString::from(self.token.as_str()))));
		}
		Err(TokenError::InvalidToken(CompactString::from(self.token.as_str())))
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
			self.token.push(c);
		}
		Ok(())
	}

	pub fn get_line(&self) -> &str {
		self.chars.get_line()
	}

	pub fn get_line_num(&self) -> usize {
		self.chars.get_line_num()
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

		let reader = BufReader::new(Cursor::new(jack_code.into_bytes()));
		let chars = CharReader::new(reader);
		let mut tokenizer = Tokenizer::new(chars);

		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Class));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("Main")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('{'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Static));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Boolean));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("test")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Void));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("main")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('{'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Var));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("SquareGame")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("game")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("game")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("SquareGame")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('.'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("new")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Do));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("game")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('.'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("run")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Do));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("game")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('.'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("dispose")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Return));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('}'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Function));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Void));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("more")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('{'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Var));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Int));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("i")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(','));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("j")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Var));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("String")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("s")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Var));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("Array")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("a")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::If));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::False));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('{'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("s")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::StrConst(CompactString::from("string constant")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("s")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Null));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("a")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('['));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::IntConst(1));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(']'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("a")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('['));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(']'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('}'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Else));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('{'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("i")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("i")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('*'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('-'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("j")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("j")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("j")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('/'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('('));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('-'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::IntConst(2));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(')'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Let));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("i")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('='));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("i")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('|'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("j")));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('}'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Keyword(Keyword::Return));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol(';'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('}'));
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Symbol('}'));
	}
}
