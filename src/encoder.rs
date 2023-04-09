use crate::parser::{Ins, DestMne, CompMne, JumpMne, SymUse};

// C-instruction format:
//
//   111 a cccccc ddd jjj
//
// a = switch bit
// c = comp bits
// d = dest bits
// j = jump bits

impl DestMne {
	fn as_u16(&self) -> u16 {
		match self {
			DestMne::DestM   => 0b111_0_000000_001_000,
			DestMne::DestD   => 0b111_0_000000_010_000,
			DestMne::DestDM  => 0b111_0_000000_011_000,
			DestMne::DestA   => 0b111_0_000000_100_000,
			DestMne::DestAM  => 0b111_0_000000_101_000,
			DestMne::DestAD  => 0b111_0_000000_110_000,
			DestMne::DestADM => 0b111_0_000000_111_000,
		}
	}
}

impl CompMne {
	fn as_u16(&self) -> u16 {
		match self {
			CompMne::Comp0       => 0b111_0_101010_000_000,
			CompMne::Comp1       => 0b111_0_111111_000_000,
			CompMne::CompMinus1  => 0b111_0_111010_000_000,
			CompMne::CompD       => 0b111_0_001100_000_000,
			CompMne::CompA       => 0b111_0_110000_000_000,
			CompMne::CompM       => 0b111_1_110000_000_000,
			CompMne::CompNotD    => 0b111_0_001101_000_000,
			CompMne::CompNotA    => 0b111_0_110001_000_000,
			CompMne::CompNotM    => 0b111_1_110001_000_000,
			CompMne::CompMinusD  => 0b111_0_001111_000_000,
			CompMne::CompMinusA  => 0b111_0_110011_000_000,
			CompMne::CompMinusM  => 0b111_1_110011_000_000,
			CompMne::CompDPlus1  => 0b111_0_011111_000_000,
			CompMne::CompAPlus1  => 0b111_0_110111_000_000,
			CompMne::CompMPlus1  => 0b111_1_110111_000_000,
			CompMne::Comp1PlusD  => 0b111_0_011111_000_000,
			CompMne::Comp1PlusA  => 0b111_0_110111_000_000,
			CompMne::Comp1PlusM  => 0b111_1_110111_000_000,
			CompMne::CompDMinus1 => 0b111_0_001110_000_000,
			CompMne::CompAMinus1 => 0b111_0_110010_000_000,
			CompMne::CompMMinus1 => 0b111_1_110010_000_000,
			CompMne::CompDPlusA  => 0b111_0_000010_000_000,
			CompMne::CompDPlusM  => 0b111_1_000010_000_000,
			CompMne::CompAPlusD  => 0b111_0_000010_000_000,
			CompMne::CompMPlusD  => 0b111_1_000010_000_000,
			CompMne::CompDMinusA => 0b111_0_010011_000_000,
			CompMne::CompDMinusM => 0b111_1_010011_000_000,
			CompMne::CompAMinusD => 0b111_0_000111_000_000,
			CompMne::CompMMinusD => 0b111_1_000111_000_000,
			CompMne::CompDAndA   => 0b111_1_000000_000_000,
			CompMne::CompDAndM   => 0b111_1_000000_000_000,
			CompMne::CompAAndD   => 0b111_1_000000_000_000,
			CompMne::CompMAndD   => 0b111_1_000000_000_000,
			CompMne::CompDOrA    => 0b111_0_010101_000_000,
			CompMne::CompDOrM    => 0b111_1_010101_000_000,
			CompMne::CompAOrD    => 0b111_0_010101_000_000,
			CompMne::CompMOrD    => 0b111_1_010101_000_000,
		}
	}
}

impl JumpMne {
	fn as_u16(&self) -> u16 {
		match self {
			JumpMne::JumpJgt => 0b111_0_000000_000_001,
			JumpMne::JumpJeq => 0b111_0_000000_000_010,
			JumpMne::JumpJge => 0b111_0_000000_000_011,
			JumpMne::JumpJlt => 0b111_0_000000_000_100,
			JumpMne::JumpJne => 0b111_0_000000_000_101,
			JumpMne::JumpJle => 0b111_0_000000_000_110,
			JumpMne::JumpJmp => 0b111_0_000000_000_111,
		}
	}
}

const A_INS_FMT: u16 = 0b0_111111111111111;

pub fn encode_ins(ins: &Ins, sym_val_table: &Vec<(u16, SymUse)>) -> Option<u16> {
	match ins {
		Ins::A1{cint} => {
			Some(A_INS_FMT & cint)
		},
		Ins::A2{sym_id} => {
			Some(A_INS_FMT & sym_val_table[*sym_id].0)
		},
		Ins::L1{..} => {
			None
		},
		Ins::C1{dest, comp} => {
			Some(dest.as_u16() | comp.as_u16())
		},
		Ins::C2{dest, comp, jump} => {
			Some(dest.as_u16() | comp.as_u16() | jump.as_u16())
		},
		Ins::C3{comp, jump} => {
			Some(comp.as_u16() | jump.as_u16())
		},
	}
}
