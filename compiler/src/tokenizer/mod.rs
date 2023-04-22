mod char_reader;
mod errors;

use compact_str::CompactString;
use lazy_static::lazy_static;
use std::io::BufRead;
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
	char_reader: CharReader<R>,
	token_buf: String,
}

impl<R: BufRead> Tokenizer<R> {
	pub fn new(char_reader: CharReader<R>) -> Self {
		Tokenizer{char_reader, token_buf: String::new()}
	}

	fn next_token(&mut self) -> Result<Option<Token>, TokenError> {
		self.token_buf.clear();
		let char_reader = &mut self.char_reader;
		while let Some(c) = char_reader.peek_char()? {
			// Eat surplus whitespace...
			if c.is_whitespace() { 
				char_reader.next_char()?;
				while let Some(c_next) = char_reader.peek_char()? {
					if c_next.is_whitespace(){ char_reader.next_char()?; } else { break; }
				}
				if self.token_buf.is_empty() { continue; } else { break; }
			}
			// Comments and symbols are not included in the current token.
			let maybe_comment = c == '/';
			let maybe_symbol = SYMBOL_SET.contains(&c);
			if (maybe_comment || maybe_symbol) && !self.token_buf.is_empty() {
				break;
			}
			// Now safe to eat the peeked char.
			char_reader.next_char()?;
			// Parse all chars in string literal inline as it may contain whitespace.
			if c == '"' {
				while let Some(c_str) = char_reader.next_char()? {
					if c_str == '"' {
						break;
					}
					self.token_buf.push(c_str);
				}
				break;
			}
			// Eat any comments (single or multiline).
			if maybe_comment {
				if let Some(c_next) = char_reader.peek_char()? {
					if c_next == '*' {
						char_reader.next_char()?;
						while let Some(c_skip) = char_reader.next_char()? {
							if let Some(c_skip_next) = char_reader.next_char()? {
								if c_skip == '*' && c_skip_next == '/' {
									break
								}
							}
						}
						continue;
					}
					else if c_next == '/' {
						char_reader.next_char()?;
						while let Some(c_skip) = char_reader.next_char()? {
							if c_skip == '\n' {
								break
							}
						}
						continue;
					}
				}
			}
			// Short-cut if we have only 1 char and it is a symbol.
			if maybe_symbol {
				return Ok(Some(Token::Symbol(c)));
			}
			self.token_buf.push(c);
		}
		// Match the token buffer against the possible tokens...
		if self.token_buf.is_empty() {
			return Ok(None);
		}
		if let Ok(x) = self.token_buf.parse::<u16>(){
			return Ok(Some(Token::IntConst(x)));
		}
		lazy_static! {
			static ref RX_STR_CONST: Regex = Regex::new(r#"".*""#).expect("RX_STR_CONST invalid!");
		}
		if RX_STR_CONST.is_match(&self.token_buf) {
			return Ok(Some(Token::StrConst(CompactString::from(self.token_buf.as_str()))));
		}
		if let Ok(keyword) = self.token_buf.parse::<Keyword>() {
			return Ok(Some(Token::Keyword(keyword)));
		}
		lazy_static! {
			static ref RX_TOKEN: Regex = Regex::new(r"[\w.$:]+").expect("RX_TOKEN invalid!");
		}
		if RX_TOKEN.is_match(&self.token_buf) {
			return Ok(Some(Token::Identifier(CompactString::from(self.token_buf.as_str()))));
		}
		Err(TokenError::InvalidToken(CompactString::from(self.token_buf.as_str())))
	}

	pub fn get_line(&self) -> &str {
		self.char_reader.get_line()
	}

	pub fn get_line_num(&self) -> usize {
		self.char_reader.get_line_num()
	}
}

impl<R: BufRead> Iterator for Tokenizer<R> {
	type Item = Result<Token, TokenError>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.next_token() {
			Ok(Some(c)) => Some(Ok(c)),
			Ok(None) => None,
			Err(e) => Some(Err(e)),
		}
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
		let char_reader = CharReader::new(reader);
		let mut tokenizer = Tokenizer::new(char_reader);

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
		assert_eq!(tokenizer.next().unwrap().unwrap(), Token::Identifier(CompactString::from("string constant")));
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
