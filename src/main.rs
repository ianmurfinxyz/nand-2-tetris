use std::collections::hash_map::{HashMap, Entry};
use std::borrow::Borrow;
use enum_iterator::Sequence;

pub const MAX_SYM_LEN: usize = 255;
pub const MAX_MNE_LEN: usize = 3;
pub const MNE_BUF_LEN: usize = MAX_MNE_LEN + 1;
pub const MAX_INT_VAL: u16 = 32767;

pub const DEFAULT_RAM_ADDRESS: u16 = 0;

pub type SymBuf = [u8; MAX_SYM_LEN];
pub type MneBuf = [u8; MNE_BUF_LEN];

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SymUse {
	ARAM,
	LROM,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MneType {
	Dest,
	Comp,
	Jump,
}

#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
pub enum DestMne {
	DestM,
	DestD,
	DestDM,
	DestA,
	DestAM,
	DestAD,
	DestADM,
}

#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
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

#[derive(Debug, PartialEq, Sequence, Clone, Copy)]
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

	#[allow(dead_code)]
	fn as_str(&self) -> &'static str {
		match self {
			DestMne::DestM   => "M",
			DestMne::DestD   => "D",
			DestMne::DestDM  => "DM",
			DestMne::DestA   => "A",
			DestMne::DestAM  => "AM",
			DestMne::DestAD  => "AD",
			DestMne::DestADM => "ADM",
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

	#[allow(dead_code)]
	fn as_str(&self) -> &'static str {
		match self {
			CompMne::Comp0       => "0",
			CompMne::Comp1       => "1",
			CompMne::CompMinus1  => "-1",
			CompMne::CompD       => "D",
			CompMne::CompA       => "A",
			CompMne::CompM       => "M",
			CompMne::CompNotD    => "!D",
			CompMne::CompNotA    => "!A",
			CompMne::CompNotM    => "!M",
			CompMne::CompMinusD  => "-D",
			CompMne::CompMinusA  => "-A",
			CompMne::CompMinusM  => "-M",
			CompMne::CompDPlus1  => "D+1",
			CompMne::CompAPlus1  => "A+1",
			CompMne::CompMPlus1  => "M+1",
			CompMne::CompDMinus1 => "D-1",
			CompMne::CompAMinus1 => "A-1",
			CompMne::CompMMinus1 => "M-1",
			CompMne::CompDPlusA  => "D+A",
			CompMne::CompDPlusM  => "D+M",
			CompMne::CompDMinusA => "D-A",
			CompMne::CompDMinusM => "D-M",
			CompMne::CompAMinusD => "A-D",
			CompMne::CompMMinusD => "M-D",
			CompMne::CompDAndA   => "D&A",
			CompMne::CompDAndM   => "D&M",
			CompMne::CompDOrA    => "D|A",
			CompMne::CompDOrM    => "D|M",
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

	#[allow(dead_code)]
	fn as_str(&self) -> &'static str {
		match self {
			JumpMne::JumpJgt => "JGT",
			JumpMne::JumpJeq => "JEQ",
			JumpMne::JumpJge => "JGE",
			JumpMne::JumpJlt => "JLT",
			JumpMne::JumpJne => "JNE",
			JumpMne::JumpJle => "JLE",
			JumpMne::JumpJmp => "JMP",
		}
	}
}

#[derive(Debug, PartialEq)]
pub enum Ins {
	A1{cint: u16},
	A2{sym_id: usize},
	L1{sym_id: usize},
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
	DuplicateLabel,
	AInsMissingArg,
	LInsMissingSym,
	LInsMissingClose,
	SymOverflow,
	IntOverflow,
	NotASCII,
	CInsNop,
}

pub type ParseResult = Result<Option<Ins>, ParseError>;

/// Parse a line of Hack assembly into its equivalent data representation. Populates the
/// symbol table as new symbols are encountered. `ins_ptr` (instruction pointer) is expected to 
/// be the current ROM address of the instruction being parsed.
///
/// # Symbols
///
/// Symbols can be either *labels* or *variables*. The former are symbollic place-holders
/// for ROM addresses, the latter for RAM addresses.
///
/// Any new symbols encountered in an A-instruction, for example ```@foo```, are added to
/// the symbol table and marked as a *variable*. Any new symbols encountered in an L-instruction, 
/// for example ```(boo)``` are added to the symbol table and marked as a *label*.
///
/// For any symbol encountered in an L-instruction which is already in the symbol table, and
/// marked as a *variable*, the mark is overriden to a *label*. *Labels* take priority because
/// *label* symbols can appear in both A and L instructions. Their presence in an L-instruction
/// identifies the symbol as a *label*, however their use in an A-instruction can appear before
/// their use in an L-instruction; below, ```foo``` is a *label* not a *variable*.
///
/// @foo
/// 0;JMP
/// (foo)
///
/// *Label* symbols are mapped immediately to the current value of `ins_ptr` (instruction pointer);
/// the current ROM address. *Variables* are all mapped to [`DEFAULT_RAM_ADDRESS`]; `parse_ins`
/// does not distribute RAM address to variables, this is a job left for the caller.
///
/// # Conflicting use of the A-register
///
/// An A-instruction ```@n``` sets the A-register, and in so doing, selects both *RAM\[n\]* and 
/// *ROM\[n\]*. Subsequent C-instructions which reference M then read/write from/to *RAM\[n\]*, 
/// and C-instruction which use a jump, jump to *ROM\[n\]*. C-instructions which do both, such
/// as ```@D=M;JMP``` have conflicting use of the A-register. Such instructions are discouraged
/// but not invalid; `parse_ins` does not restrict their use.
///
/// # Example
///
/// ```
/// let mut sym_key_table = HashMap::new();
/// let mut sym_val_table = vec![];
/// assert_eq!(parse_ins("@123", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 123})));
/// assert_eq!(parse_ins("#comment\n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
/// ```
pub fn parse_ins(line: &str, ins_ptr: u16, sym_key_table: &mut HashMap<String, usize>,
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
					_ => {
						push_mne_char(c, &mut mb0, &mut mi0, None)?;
						dfa = DFA::CFirst;
					}
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
			let cint = match unsafe {std::str::from_utf8_unchecked(&sb0[..si0])}.parse::<u16>() {
				Ok(i) => i,
				Err(_) => return Err(ParseError::IntOverflow),
			};
			if cint > MAX_INT_VAL {
				return Err(ParseError::IntOverflow)
			}
			Ok(Some(Ins::A1{cint}))
		},
		DFA::ASym => {
			let sym = unsafe { std::str::from_utf8_unchecked(&sb0[..si0]) };
			let sym_id = match sym_key_table.entry(String::from(sym.borrow())) {
				Entry::Occupied(entry) => {
					*entry.get()
				},
				Entry::Vacant(entry) => {
					let sym_id = sym_val_table.len();
					sym_val_table.push((DEFAULT_RAM_ADDRESS, SymUse::ARAM));
					*entry.insert(sym_id)
				},
			};
			Ok(Some(Ins::A2{sym_id}))
		},
		DFA::LFirst => {
			Err(ParseError::LInsMissingSym)
		},
		DFA::LRest => {
			Err(ParseError::LInsMissingClose)
		},
		DFA::LClose => {
			let sym = unsafe { std::str::from_utf8_unchecked(&sb0[..si0]) };
			let sym_val = (ins_ptr + 1, SymUse::LROM);
			let sym_id = match sym_key_table.entry(String::from(sym.borrow())) {
				Entry::Occupied(entry) => {
					let sym_id = *entry.get();
					if sym_val_table[sym_id].1 == SymUse::LROM {
						return Err(ParseError::DuplicateLabel)
					}
					sym_val_table[sym_id] = sym_val;
					sym_id
				},
				Entry::Vacant(entry) => {
					let sym_id = sym_val_table.len();
					sym_val_table.push(sym_val);
					*entry.insert(sym_id)
				},
			};
			Ok(Some(Ins::L1{sym_id}))
		},
		DFA::CFirst => {
			Err(ParseError::CInsNop)
		},
		DFA::CComp => {
			let dest = DestMne::from_mne_buf(mb0)?;
			let comp = CompMne::from_mne_buf(mb1)?;
			Ok(Some(Ins::C1{dest, comp}))
		},
		DFA::CJump1 => {
			let comp = CompMne::from_mne_buf(mb0)?;
			let jump = JumpMne::from_mne_buf(mb1)?;
			Ok(Some(Ins::C3{comp, jump}))
		},
		DFA::CJump2 => {
			let dest = DestMne::from_mne_buf(mb0)?;
			let comp = CompMne::from_mne_buf(mb1)?;
			let jump = JumpMne::from_mne_buf(mb2)?;
			Ok(Some(Ins::C2{dest, comp, jump}))
		},
	}
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use super::*;

	#[test]
	fn test_blank_and_comment_lines(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Blank lines should be reported as a valid but empty result.
		assert_eq!(parse_ins("", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
		assert_eq!(parse_ins("\n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
		assert_eq!(parse_ins("		\n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
		assert_eq!(parse_ins("        \n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));

		// Comment-only lines should be reported as a valid but empty result.
		assert_eq!(parse_ins("#        \n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
		assert_eq!(parse_ins("#comment", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));
		assert_eq!(parse_ins("#comment\n", 0, &mut sym_key_table, &mut sym_val_table), Ok(None));

		// Blank and comment lines should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_ains_int_parsing(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Well formed integers should be correctly parsed.
		assert_eq!(parse_ins("@0", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 0})));
		assert_eq!(parse_ins("@1234", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 1234})));
		assert_ne!(parse_ins("@1234", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 4321})));
		assert_eq!(parse_ins("@32767", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 32767})));

		// Malformed a-ins with missing args should be detected.
		assert_eq!(parse_ins("@", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::AInsMissingArg));

		// Overflows of Hack RAM/ROM should be detected.
		assert_eq!(parse_ins("@32768", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));
		assert_eq!(parse_ins("@999999", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));

		// Whitespace should be ignored.
		assert_eq!(parse_ins("@3 2 7 6 7", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 32767})));
		assert_eq!(parse_ins("@3	27 6 7", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 32767})));
		assert_eq!(parse_ins("@9 9 9 9 9 9", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));

		// Comments should be ignored.
		assert_eq!(parse_ins("@1#234", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 1})));
		assert_eq!(parse_ins("@12    #@34", 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 12})));

		// Max symbol length integer should be detected as an int overflow (not overflow the symbol buffer).
		let sym_limit_int = "@".to_string() + "9".repeat(MAX_SYM_LEN).borrow();
		assert_eq!(parse_ins(&sym_limit_int, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::IntOverflow));

		// Overflowing the symbol buffer should be detected.
		let sym_overflow_int = "@".to_string() + "9".repeat(MAX_SYM_LEN + 1).borrow();
		assert_eq!(parse_ins(&sym_overflow_int, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::SymOverflow));

		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_ains_symbol_table_population(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		const TEST_SIZE: usize = 20;

		for i in 0..TEST_SIZE {
			let var = format!("var{}", i);
			let ins = format!("@{}", var);

			// Each existing symbol encountered should not declare a new variable.
			for _repeat in 0..3 {

				// Each new symbol encountered should declare a new variable.
				assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A2{sym_id: i})));

				// Mapped value of hash map should be the correct index into the value table.
				assert_eq!(sym_key_table.get_key_value(&var), Some((&var, &i)));

				// New variables should be assigned the default ram address and be correctly
				// identified as being used to store RAM addresses.
				assert_eq!(sym_val_table[i], (DEFAULT_RAM_ADDRESS, SymUse::ARAM));
			}
		}

		// Repeats should not of populated new variables.
		assert!(sym_key_table.len() == TEST_SIZE);

		// A value should exist for every symbol.
		assert!(sym_val_table.len() == TEST_SIZE);
	}

	#[test]
	fn test_malformed_ains(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Malformed symbols (or malformed integers) should be detected.
		assert_eq!(parse_ins("@4foo", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::ExpectedDigit{found: 'f', pos: 2}));

		// Erroneous a-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_lins_symbol_table_population(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		const TEST_SIZE: usize = 20;

		for sym_id in 0..TEST_SIZE {
			let sym = format!("label{}", sym_id);
			let ins = format!("({})", sym);

			let ins_ptr = 0u16;

			// Each new symbol encountered should declare a new label.
			assert_eq!(parse_ins(&ins, ins_ptr, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::L1{sym_id})));

			// Mapped value of hash map should be the correct index into the value table.
			assert_eq!(sym_key_table.get_key_value(&sym), Some((&sym, &sym_id)));

			// New labels should be assigned the next ROM address and be correctly
			// identified as being used to store ROM addresses.
			assert_eq!(sym_val_table[sym_id], (ins_ptr + 1, SymUse::LROM));

			// Duplication errors should be robust in the face of multiple detections.
			for _repeat in 0..3 {

				// Duplicate labels should be identified as an error (cannot jump to 2 instructions).
				assert_eq!(parse_ins(&ins, ins_ptr, &mut sym_key_table, &mut sym_val_table), Err(ParseError::DuplicateLabel));
			}
		}

		// Duplication errors should not of populated new variables.
		assert!(sym_key_table.len() == TEST_SIZE);

		// A value should exist for every symbol.
		assert!(sym_val_table.len() == TEST_SIZE);
	}

	#[test]
	fn test_malformed_lins(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// L-instructions with no symbol should be detected.
		assert_eq!(parse_ins("(", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::LInsMissingSym));

		// Unexpected symbol after '(' should be detected.
		assert_eq!(parse_ins("()", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::ExpectedFirstSymChar{found: ')', pos: 1}));
		assert_eq!(parse_ins("(-", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::ExpectedFirstSymChar{found: '-', pos: 1}));
		assert_eq!(parse_ins("(+", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::ExpectedFirstSymChar{found: '+', pos: 1}));

		// Malformed symbols which start with a digit should be detected.
		assert_eq!(parse_ins("(4foo", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::ExpectedFirstSymChar{found: '4', pos: 1}));

		// L-instructions with no close should be detected.
		assert_eq!(parse_ins("(foo", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::LInsMissingClose));

		// Erroneous l-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_mixed_symbol_table_population(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		let var_num = 0usize;
		let mut ins_ptr = 0u16;

		// Symbol foo is new so should be assumed to be a variable.
		assert_eq!(parse_ins("@foo", ins_ptr, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A2{sym_id: var_num})));
		assert_eq!(sym_key_table.get("foo"), Some(&var_num));
		assert_eq!(sym_val_table.len(), var_num + 1);
		assert_eq!(sym_val_table[var_num], (DEFAULT_RAM_ADDRESS, SymUse::ARAM));

		ins_ptr += 1;

		// Label using symbol foo is encountered; foo should now be overriden to a label.
		assert_eq!(parse_ins("(foo)", ins_ptr, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::L1{sym_id: var_num})));
		assert_eq!(sym_key_table.get("foo"), Some(&var_num));
		assert_eq!(sym_val_table.len(), var_num + 1);

		let foo_ins_ptr = ins_ptr + 1;
		assert_eq!(sym_val_table[var_num], (foo_ins_ptr, SymUse::LROM));

		ins_ptr += 1;

		// Symbol foo is old, and a label, and should continue to be identified as such.
		assert_eq!(parse_ins("@foo", ins_ptr, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A2{sym_id: var_num})));
		assert_eq!(sym_key_table.get("foo"), Some(&var_num));
		assert_eq!(sym_val_table.len(), var_num + 1);
		assert_eq!(sym_val_table[var_num], (foo_ins_ptr, SymUse::LROM));
	}

	use enum_iterator::all;

	#[test]
	fn test_c1ins_parsing(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// All permutations of dest=comp should be correctly parsed (and are valid).
		for dest in all::<DestMne>().collect::<Vec<_>>() {
			for comp in all::<CompMne>().collect::<Vec<_>>() {
				let ins = format!("{}={}", dest.as_str(), comp.as_str());
				assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::C1{dest, comp})));
			}
		}

		// C-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_c2ins_parsing(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// All permutations of dest=comp;jump should be correctly parsed (and are valid).
		for dest in all::<DestMne>().collect::<Vec<_>>() {
			for comp in all::<CompMne>().collect::<Vec<_>>() {
				for jump in all::<JumpMne>().collect::<Vec<_>>() {
					let ins = format!("{}={};{}", dest.as_str(), comp.as_str(), jump.as_str());
					assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::C2{dest, comp, jump})));
				}
			}
		}

		// C-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_c3ins_parsing(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// All permutations of comp;jump should be correctly parsed (and are valid).
		for comp in all::<CompMne>().collect::<Vec<_>>() {
			for jump in all::<JumpMne>().collect::<Vec<_>>() {
				let ins = format!("{};{}", comp.as_str(), jump.as_str());
				assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::C3{comp, jump})));
			}
		}

		// C-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_unknown_cins_error(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Jibberish dest should be detected as unknown.
		let mut mne_type = Some(MneType::Dest);
		let mut mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, ' ' as u8];
		let mut ins = format!("jib={}", CompMne::CompNotD.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Long jibberish dest should be detected as unknown.
		mne_type = None;
		mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, 'b' as u8];
		ins = format!("jibberish={}", CompMne::CompNotD.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Jibberish comp should be detected as unknown.
		mne_type = Some(MneType::Comp);
		mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, ' ' as u8];
		ins = format!("{}=jib", DestMne::DestD.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Long jibberish comp should be detected as unknown.
		mne_type = Some(MneType::Comp);
		mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, 'b' as u8];
		ins = format!("{}=jibberish", DestMne::DestD.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Jibberish jump should be detected as unknown.
		mne_type = Some(MneType::Jump);
		mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, ' ' as u8];
		ins = format!("{}={};jib", DestMne::DestD.as_str(), CompMne::CompM.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Long jibberish jump should be detected as unknown.
		mne_type = Some(MneType::Jump);
		mne_buf = ['j' as u8, 'i' as u8, 'b' as u8, 'b' as u8];
		ins = format!("{}={};jibberish", DestMne::DestD.as_str(), CompMne::CompM.as_str());
		assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::UnknownMne{mne_type, mne_buf}));

		// Erroneous c-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_nop_cins(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Stand-along comp c-instructions have no effect and should be detected.
		for comp in all::<CompMne>().collect::<Vec<_>>() {
			let ins = format!("{}", comp.as_str());
			assert_eq!(parse_ins(&ins, 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::CInsNop));
		}

		// Erroneous c-instructions should populate no symbols.
		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}

	#[test]
	fn test_unicode_not_supported(){
		let mut sym_key_table = HashMap::new();
		let mut sym_val_table = vec![];

		// Unicode is not supported and should be detected.
		assert_eq!(parse_ins("语言处理", 0, &mut sym_key_table, &mut sym_val_table), Err(ParseError::NotASCII));

		assert!(sym_key_table.is_empty());
		assert!(sym_val_table.is_empty());
	}
}


fn main() {
	let mut sym_key_table = HashMap::new();
	let mut sym_val_table = vec![];
	assert_eq!(parse_ins("@1234", 0u16, &mut sym_key_table, &mut sym_val_table), Ok(Some(Ins::A1{cint: 1234})));
}
