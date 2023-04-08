use std::collections::hash_map::{HashMap, Entry};
use std::borrow::Borrow;

pub const MAX_SYM_LEN: usize = 255;
pub const MAX_MNE_LEN: usize = 3;
pub const MNE_BUF_LEN: usize = MAX_MNE_LEN + 1;
pub const MAX_INT_VAL: u16 = 32767;

const DEFAULT_RAM_ADDRESS: u16 = 0;

pub type SymBuf = [u8; MAX_SYM_LEN];
pub type MneBuf = [u8; MNE_BUF_LEN];

#[derive(Debug, PartialEq)]
pub enum SymUse {
	ARAM,
	LROM,
}

#[derive(Debug, PartialEq)]
pub enum MneType {
	Dest,
	Comp,
	Jump,
}

#[derive(Debug, PartialEq)]
pub enum DestMne {
	DestM,
	DestD,
	DestDM,
	DestA,
	DestAM,
	DestAD,
	DestADM,
}

#[derive(Debug, PartialEq)]
pub enum CompMne {
	Comp0,
	Comp1,
	CompMinus1,
	CompD,
	CompA,
	CompM,
	CompNotD,
	CompNotA,
	CompNotM,
	CompMinusD,
	CompMinusA,
	CompMinusM,
	CompDPlus1,
	CompAPlus1,
	CompMPlus1,
	CompDMinus1,
	CompAMinus1,
	CompMMinus1,
	CompDPlusA,
	CompDPlusM,
	CompDMinusA,
	CompDMinusM,
	CompAMinusD,
	CompMMinusD,
	CompDAndA,
	CompDAndM,
	CompDOrA,
	CompDOrM,
}

#[derive(Debug, PartialEq)]
pub enum JumpMne {
	JumpJgt,
	JumpJeq,
	JumpJge,
	JumpJlt,
	JumpJne,
	JumpJle,
	JumpJmp,
}

impl DestMne {
	fn from_mne_buf(mne_buf: MneBuf) -> Result<DestMne, ParseError> {
		let mne_str = unsafe {
			std::str::from_utf8_unchecked(mne_buf.as_ref())
		};
		match mne_str {
			"M   " => Ok(DestMne::DestM),
			"D   " => Ok(DestMne::DestD),
			"DM  " => Ok(DestMne::DestDM),
			"A   " => Ok(DestMne::DestA),
			"AM  " => Ok(DestMne::DestAM),
			"AD  " => Ok(DestMne::DestAD),
			"ADM " => Ok(DestMne::DestADM),
			_      => Err(ParseError::UnknownMne{mne_type: Some(MneType::Dest), mne_buf}),
		}
	}
}

impl CompMne {
	fn from_mne_buf(mne_buf: MneBuf) -> Result<CompMne, ParseError> {
		let mne_str = unsafe {
			std::str::from_utf8_unchecked(mne_buf.as_ref())
		};
		match mne_str {
			"0   " => Ok(CompMne::Comp0),
			"1   " => Ok(CompMne::Comp1),
			"-1  " => Ok(CompMne::CompMinus1),
			"D   " => Ok(CompMne::CompD),
			"A   " => Ok(CompMne::CompA),
			"M   " => Ok(CompMne::CompM),
			"!D  " => Ok(CompMne::CompNotD),
			"!A  " => Ok(CompMne::CompNotA),
			"!M  " => Ok(CompMne::CompNotM),
			"-D  " => Ok(CompMne::CompMinusD),
			"-A  " => Ok(CompMne::CompMinusA),
			"-M  " => Ok(CompMne::CompMinusM),
			"D+1 " => Ok(CompMne::CompDPlus1),
			"A+1 " => Ok(CompMne::CompAPlus1),
			"M+1 " => Ok(CompMne::CompMPlus1),
			"D-1 " => Ok(CompMne::CompDMinus1),
			"A-1 " => Ok(CompMne::CompAMinus1),
			"M-1 " => Ok(CompMne::CompMMinus1),
			"D+A " => Ok(CompMne::CompDPlusA),
			"D+M " => Ok(CompMne::CompDPlusM),
			"D-A " => Ok(CompMne::CompDMinusA),
			"D-M " => Ok(CompMne::CompDMinusM),
			"A-D " => Ok(CompMne::CompAMinusD),
			"M-D " => Ok(CompMne::CompMMinusD),
			"D&A " => Ok(CompMne::CompDAndA),
			"D&M " => Ok(CompMne::CompDAndM),
			"D|A " => Ok(CompMne::CompDOrA),
			"D|M " => Ok(CompMne::CompDOrM),
			_      => Err(ParseError::UnknownMne{mne_type: Some(MneType::Comp), mne_buf}),
		}
	}
}

impl JumpMne {
	fn from_mne_buf(mne_buf: MneBuf) -> Result<JumpMne, ParseError> {
		let mne_str = unsafe {
			std::str::from_utf8_unchecked(mne_buf.as_ref())
		};
		match mne_str {
			"JGT " => Ok(JumpMne::JumpJgt),
			"JEQ " => Ok(JumpMne::JumpJeq),
			"JGE " => Ok(JumpMne::JumpJge),
			"JLT " => Ok(JumpMne::JumpJlt),
			"JNE " => Ok(JumpMne::JumpJne),
			"JLE " => Ok(JumpMne::JumpJle),
			"JMP " => Ok(JumpMne::JumpJmp),
			_      => Err(ParseError::UnknownMne{mne_type: Some(MneType::Jump), mne_buf}),
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Ins {
	A1{c_int: u16},
	A2{i_sym: usize},
	L1{i_sym: usize},
	C1{dest: DestMne, comp: CompMne},
	C2{dest: DestMne, comp: CompMne, jump: JumpMne},
	C3{comp: CompMne, jump: JumpMne},
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
	UnknownMne{mne_type: Option<MneType>, mne_buf: MneBuf},
	ExpectedFirstSymChar{found: char, pos: usize},
	ExpectedSymChar{found: char, pos: usize},
	ExpectedDigit{found: char, pos: usize},
	UnexpectedChar{found: char, pos: usize},
	AInsMissingArg,
	LInsMissingSym,
	LInsMissingClose,
	CInsNoEffect,
	SymOverflow,
	IntOverflow,
	NotASCII,
}

pub type ParseResult = Result<Option<Ins>, ParseError>;

fn parse_ins(line: &str, ins_ptr: u16, sym_key_table: &mut HashMap<String, usize>,
	sym_val_table: &mut Vec<(u16, SymUse)>) -> ParseResult {

	enum DFA {
		Start,
		AOpen,
		ASym,
		AInt,
		LFirst,
		LClose,
		LRest,
		CFirst,
		CComp,
		CJump1,
		CJump2,
	}

	if !line.is_ascii() {
		return Err(ParseError::NotASCII)
	}

	let mut dfa = DFA::Start;

	let mne_buf_new = ||[' ' as u8; MNE_BUF_LEN];
	let sym_buf_new = ||[' ' as u8; MAX_SYM_LEN];

	let mut sb0 = sym_buf_new();
	let mut mb0 = mne_buf_new();
	let mut mb1 = mne_buf_new();
	let mut mb2 = mne_buf_new();

	let mut si0 = 0usize;
	let mut mi0 = 0usize;
	let mut mi1 = 0usize;
	let mut mi2 = 0usize;

	fn push_sym_char(c: char, sb: &mut SymBuf, si: &mut usize) -> Result<(), ParseError> {
		if *si == sb.len() {
			return Err(ParseError::SymOverflow);
		}
		sb[*si] = c as u8;
		*si += 1;
		Ok(())
	}

	fn push_mne_char(c: char, mb: &mut MneBuf, mi: &mut usize, mne_type: Option<MneType>) -> Result<(), ParseError> {
		mb[*mi] = c as u8;
		*mi += 1;
		if *mi > MAX_MNE_LEN {
			return Err(ParseError::UnknownMne{mne_type, mne_buf: *mb});
		}
		Ok(())
	}

	for (pos, c) in line.char_indices() {
		if c.is_whitespace() {
			continue;
		}
		if c == '#' {
			break;
		}
		match dfa {
			DFA::Start => {
				match c {
					'@' => dfa = DFA::AOpen,
					'(' => dfa = DFA::LFirst,
					_ => dfa = DFA::CFirst,
				}
			},
			DFA::AOpen => {
				match c {
					'0'..='9' => {
						dfa = DFA::AInt;
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					'_'|'.'|'$'|':'|'a'..='z'|'A'..='Z' => {
						dfa = DFA::ASym;
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					_ => return Err(ParseError::ExpectedFirstSymChar{found: c, pos})
				}
			},
			DFA::ASym => {
				match c {
					'_'|'.'|'$'|':'|'a'..='z'|'A'..='Z'|'0'..='9' => {
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					_ => return Err(ParseError::ExpectedSymChar{found: c, pos})
				}
			},
			DFA::AInt => {
				match c {
					'0'..='9' => {
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					_ => return Err(ParseError::ExpectedDigit{found: c, pos})
				}
			},
			DFA::LFirst => {
				match c {
					'_'|'.'|'$'|':'|'a'..='z'|'A'..='Z' => {
						dfa = DFA::LRest;
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					_ => return Err(ParseError::ExpectedFirstSymChar{found: c, pos})
				}
			},
			DFA::LRest => {
				match c {
					'_'|'.'|'$'|':'|'a'..='z'|'A'..='Z'|'0'..='9' => {
						push_sym_char(c, &mut sb0, &mut si0)?;
					},
					')' => {
						dfa = DFA::LClose;
					},
					_ => return Err(ParseError::ExpectedSymChar{found: c, pos})
				}
			},
			DFA::LClose => {
				return Err(ParseError::UnexpectedChar{found: c, pos})
			},
			DFA::CFirst => {
				match c {
					';' => dfa = DFA::CJump1,
					'=' => dfa = DFA::CComp,
					_ => push_mne_char(c, &mut mb0, &mut mi0, None)?,
				}
			},
			DFA::CComp => {
				match c {
					';' => dfa = DFA::CJump2,
					_ => push_mne_char(c, &mut mb1, &mut mi1, Some(MneType::Comp))?,
				}
			},
			DFA::CJump1 => {
				push_mne_char(c, &mut mb1, &mut mi1, Some(MneType::Jump))?;
			},
			DFA::CJump2 => {
				push_mne_char(c, &mut mb2, &mut mi2, Some(MneType::Jump))?;
			},
		}
	}

	match dfa {
		DFA::Start => {
			Ok(None)
		},
		DFA::AOpen => {
			Err(ParseError::AInsMissingArg)
		},
		DFA::AInt => {
			let c_int = match unsafe {std::str::from_utf8_unchecked(&sb0[..si0])}.parse::<u16>() {
				Ok(i) => i,
				Err(_) => return Err(ParseError::IntOverflow),
			};
			if c_int > MAX_INT_VAL {
				return Err(ParseError::IntOverflow)
			}
			Ok(Some(Ins::A1{c_int}))
		},
		DFA::ASym => {
			let sym = unsafe {
				std::str::from_utf8_unchecked(&sb0[..si0])
			};
			let i_sym = match sym_key_table.entry(String::from(sym.borrow())) {
				Entry::Occupied(entry) => {
					*entry.get()
				},
				Entry::Vacant(entry) => {
					let i_sym = sym_val_table.len();
					sym_val_table.push((DEFAULT_RAM_ADDRESS, SymUse::ARAM));
					*entry.insert(i_sym)
				},
			};
			Ok(Some(Ins::A2{i_sym}))
		},
		DFA::LFirst => {
			Err(ParseError::LInsMissingSym)
		},
		DFA::LRest => {
			Err(ParseError::LInsMissingClose)
		},
		DFA::LClose => {
			let sym = unsafe {
				std::str::from_utf8_unchecked(&sb0[..si0])
			};
			let i_sym = match sym_key_table.entry(String::from(sym.borrow())) {
				Entry::Occupied(entry) => {
					*entry.get()
				},
				Entry::Vacant(entry) => {
					let i_sym = sym_val_table.len();
					sym_val_table.push((0, SymUse::LROM));
					*entry.insert(i_sym)
				},
			};
			sym_val_table[i_sym] = (ins_ptr + 1, SymUse::LROM);
			Ok(Some(Ins::L1{i_sym}))
		},
		DFA::CFirst => {
			Err(ParseError::CInsNoEffect)
		},
		DFA::CComp => {
			let dest = DestMne::from_mne_buf(mb0)?;
			let comp = CompMne::from_mne_buf(mb1)?;
			Ok(Some(Ins::C1{dest, comp}))
		},
		DFA::CJump1 => {
			let dest = DestMne::from_mne_buf(mb0)?;
			let comp = CompMne::from_mne_buf(mb1)?;
			let jump = JumpMne::from_mne_buf(mb2)?;
			Ok(Some(Ins::C2{dest, comp, jump}))
		},
		DFA::CJump2 => {
			let comp = CompMne::from_mne_buf(mb0)?;
			let jump = JumpMne::from_mne_buf(mb1)?;
			Ok(Some(Ins::C3{comp, jump}))
		},
	}
}

// tests to do:
// - happy path tests; expected results
// - error path tests; expected errors
// - full program tests; expected machine code
// - junk data tests; should handle junk data
// - limits tests; symbols that are at size limit, over size limit, 1 less etc

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use super::*;

	#[test]
	fn test_int_ains(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		assert_eq!(parse_ins("@1234", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{c_int: 1234})));
		assert_ne!(parse_ins("@1234", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{c_int: 4321})));
		assert_eq!(parse_ins("@32767", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{c_int: 32767})));
		assert_eq!(parse_ins("@", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::AInsMissingArg));
		assert_eq!(parse_ins("@32768", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));
		assert_eq!(parse_ins("@999999", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));

		let sym_limit_int = "@".to_string() + "9".repeat(MAX_SYM_LEN).borrow();
		assert_eq!(parse_ins(&sym_limit_int, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));

		let sym_overflow_int = "@".to_string() + "9".repeat(MAX_SYM_LEN + 1).borrow();
		assert_eq!(parse_ins(&sym_overflow_int, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::SymOverflow));

		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	//fn test_sym_ains(){
		//let mut sym_key_table = HashMap::new();
		//let mut sym_val_table = vec![];

		//assert_eq!(parse_ins("@1234", 0u16, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{c_int: 1234})));

		//assert_eq!(parse_ins("@weed", 0u16, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A2{i_sym: 0})));
		//assert!(sym_key_table.get_key_value(0));
	//}


}


fn main() {
	let mut sym_key_table = HashMap::new();
	let mut sym_val_table = vec![];
	assert_eq!(parse_ins("@1234", 0u16, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{c_int: 1234})));
}
