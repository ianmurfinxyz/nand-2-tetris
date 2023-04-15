use std::io::{self, Write};
use core::ops::Range;
use compact_str::CompactString;
use crate::tokenizer::*;
use crate::parser::*;

const CALL_STACK_BASE_ADDRESS: u16 = 256;
const TEMP_SEGMENT_BASE_ADDRESS: u16 = 5;
const MAX_STATIC_VARIABLES: usize = 240;
const EQ_IMPL_LABEL: &'static str = "EQ_IMPL";
const GT_IMPL_LABEL: &'static str = "GT_IMPL";
const LT_IMPL_LABEL: &'static str = "LT_IMPL";
const RETURN_IMPL_LABEL: &'static str = "RETURN_IMPL";
const CALL_IMPL_LABEL: &'static str = "CALL_IMPL";
const ENTRY_IMPL_LABEL: &'static str = "ENTRY_IMPL";

pub enum CodeError {
	IndexOutOfBounds{segment: VmSeg, index: u16, bounds: Range<usize>},
	IoError(io::Error),
}

impl From<io::Error> for CodeError {
	fn from(e: io::Error) -> Self {
		CodeError::IoError(e)
	}
}

pub struct Coder {
	call_count: usize,
	eq_count: usize,
	lt_count: usize,
	gt_count: usize,
}

pub struct InsContext {
	vm_file_name: String,
	vm_function_name: String,
}

impl Coder {
	pub fn new() -> Self {
		Coder{call_count: 0, eq_count: 0, lt_count: 0, gt_count: 0}
	}

	pub fn write_core_impl<W: Write>(out: &mut W) -> std::io::Result<()> {
		let bootstrap_impl = format!("\
			@{}
			D=A
			@SP
			M=D
			@{}
			0;JMP
		", CALL_STACK_BASE_ADDRESS, ENTRY_IMPL_LABEL);
		let eq_impl = format!("\
			({})
			@R15
			M=D
			@SP
			AM=M-1
			D=M
			A=A-1
			D=M-D
			M=0
			@END_EQ
			D;JNE
			@SP
			A=M-1
			M=-1
			(END_EQ)
			@R15
			A=M
			0;JMP
		", EQ_IMPL_LABEL);
		let gt_impl = format!("\
			({})
			@R15
			M=D
			@SP
			AM=M-1
			D=M
			A=A-1
			D=M-D
			M=0
			@END_GT
			D;JLE
			@SP
			A=M-1
			M=-1
			(END_GT)
			@R15
			A=M
			0;JMP
		", GT_IMPL_LABEL);
		let lt_impl = format!("\
			({})
			@R15
			M=D
			@SP
			AM=M-1
			D=M
			A=A-1
			D=M-D
			M=0
			@END_LT
			D;JGE
			@SP
			A=M-1
			M=-1
			(END_LT)
			@R15
			A=M
			0;JMP
		", LT_IMPL_LABEL);
		let return_impl = format!("\
			({})
			@5
			D=A
			@LCL
			A=M-D
			D=M
			@R13
			M=D
			@SP
			AM=M-1
			D=M
			@ARG
			A=M
			M=D
			D=A
			@SP
			M=D+1
			@LCL
			D=M
			@R14
			AM=D-1
			D=M
			@THAT
			M=D
			@R14
			AM=M-1
			D=M
			@THIS
			M=D
			@R14
			AM=M-1
			D=M
			@ARG
			M=D
			@R14
			AM=M-1
			D=M
			@LCL
			M=D
			@R13
			A=M
			0;JMP
		", RETURN_IMPL_LABEL);
		let call_impl = format!("\
			({})
			@SP
			A=M
			M=D
			@LCL
			D=M
			@SP
			AM=M+1
			M=D
			@ARG
			D=M
			@SP
			AM=M+1
			M=D
			@THIS
			D=M
			@SP
			AM=M+1
			M=D
			@THAT
			D=M
			@SP
			AM=M+1
			M=D
			@4
			D=A
			@R13
			D=D+M
			@SP
			D=M-D
			@ARG
			M=D
			@SP
			MD=M+1
			@LCL
			M=D
			@R14
			A=M
			0;JMP
		", CALL_IMPL_LABEL);
		let entry_impl = format!("\
			({})
			@0
			D=A
			@R13
			M=D
			@sys.init
			D=A
			@R14
			M=D
			@RET_ADDRESS_SYS_INIT
			D=A
			@95
			0;JMP
			(RET_ADDRESS_SYS_INIT)
		", ENTRY_IMPL_LABEL);
	
		write!(out, "{}", bootstrap_impl)?;
		write!(out, "{}", eq_impl)?;
		write!(out, "{}", gt_impl)?;
		write!(out, "{}", lt_impl)?;
		write!(out, "{}", return_impl)?;
		write!(out, "{}", call_impl)?;
		write!(out, "{}", entry_impl)?;
	
		Ok(())
	}

	pub fn write_vm_ins<W: Write>(&mut self, out: &mut W, vm_ins: VmIns, ctx: &InsContext) -> Result<(), CodeError> {
		return match vm_ins {
			VmIns::Function{name, locals_count} => write_function_ins(out, ctx, name, locals_count),
			VmIns::Call{function, args_count} => {self.call_count += 1; write_call_ins(out, ctx, function, args_count, self.call_count)},
			VmIns::Push{segment, index} => write_push_ins(out, ctx, segment, index),
			VmIns::Pop{segment, index} => write_pop_ins(out, ctx, segment, index),
			VmIns::Label{label} => write_label_ins(out, ctx, label),
			VmIns::IfGoto{label} => write_if_goto_ins(out, ctx, label),
			VmIns::Goto{label} => write_goto_ins(out, ctx, label),
			VmIns::Return => write_return_ins(out),
			VmIns::Add => write_add_ins(out),
			VmIns::Sub => write_sub_ins(out),
			VmIns::Neg => write_neg_ins(out),
			VmIns::And => write_and_ins(out),
			VmIns::Or => write_or_ins(out),
			VmIns::Not => write_not_ins(out),
			VmIns::Eq => {self.eq_count += 1; write_eq_ins(out, self.eq_count)},
			VmIns::Lt => {self.lt_count += 1; write_lt_ins(out, self.lt_count)},
			VmIns::Gt => {self.gt_count += 1; write_gt_ins(out, self.gt_count)},
		};
	
		fn write_function_ins<W: Write>(out: &mut W, ctx: &InsContext, name: CompactString, locals_count: u16) -> Result<(), CodeError> {
			debug_assert_eq!(name, ctx.vm_function_name);
			match locals_count {
				0 => {
					write!(out, "\
						({}.{})
					", ctx.vm_file_name, name)?;
				},
				1 => {
					write!(out, "\
						({}.{})
						@SP
						AM=M+1
						A=A-1
						M=0
					", ctx.vm_file_name, name)?;
				},
				2 => {
					write!(out, "\
						({}.{})
						@SP
						AM=M+1
						A=A-1
						M=0
						@SP
						AM=M+1
						A=A-1
						M=0
					", ctx.vm_file_name, name)?;
				},
				_ => {
					write!(out, "\
						({}.{})
						@{}
						D=A
						(LOOP_{}.{})
						D=D-1
						@SP
						AM=M+1
						A=A-1
						M=0
						@LOOP_{}.{}
						D;JGT
					", ctx.vm_file_name, name, locals_count, ctx.vm_file_name, name, ctx.vm_file_name, name)?;
				},
			};
			Ok(())
		}
	
		fn write_call_ins<W: Write>(out: &mut W, ctx: &InsContext, function: CompactString, args_count: u16, call_count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@{}
				D=A
				@R13
				M=D
				@{}.{}
				D=A
				@R14 
				M=D
				@{}.{}$ret.{}
				D=A
				@{}
				0;JMP
			", args_count, ctx.vm_file_name, function, ctx.vm_file_name, function, call_count, CALL_IMPL_LABEL)?;
			Ok(())
		}
	
		fn write_push_ins<W: Write>(out: &mut W, ctx: &InsContext, segment: VmSeg, index: u16) -> Result<(), CodeError> {
			let label = compose_segment_label(ctx, segment, index)?;
			match segment {
				VmSeg::Constant => {
					match index {
						0 => {
							write!(out, "\
								@SP
								M=M+1
								A=M-1
								M=0
							")?;
						},
						1 => {
							write!(out, "\
								@SP
								M=M+1
								A=M-1
								M=1
							")?;
						},
						_ => { 
							write!(out, "\
								@{}
								D=A
								@SP
								M=M+1
								A=M-1
								M=D
							", index)?;
						},
					}
				},
				VmSeg::Static => {
					write!(out, "\
						@{}
						D=M
						@SP
						AM=M+1
						A=A-1
						M=D
					", label)?;
				},
				_ => {
					match index {
						0 => {
							write!(out, "\
								@{}
								A=M
								D=M
								@SP
								AM=M+1
								A=A-1
								M=D
							", label)?;
						},
						1 => {
							write!(out, "\
								@{}
								A=M+1
								D=M
								@SP
								AM=M+1
								A=A-1
								M=D
							", label)?;
						},
						_ => { 
							write!(out, "\
								@{}
								D=A
								@{}
								A=M+D
								D=M
								@SP
								AM=M+1
								A=A-1
								M=D
							", index, label)?;
						},
					};
				}
			};
			Ok(())
		}
	
		fn write_pop_ins<W: Write>(out: &mut W, ctx: &InsContext, segment: VmSeg, index: u16) -> Result<(), CodeError> {
			let label = compose_segment_label(ctx, segment, index)?;
			match segment {
				VmSeg::Constant => (), // NOP
				VmSeg::Static => {
					write!(out, "\
						@SP
						M=M-1
						A=M
						D=M
						@{}
						M=D
					", label)?;
				},
				_ => {
					match index {
						0 => {
							write!(out, "\
								@SP
								M=M-1
								A=M
								D=M
								@{}
								D=D+M
								@SP
								A=M
								A=M
								A=D-A
								M=D-A
							", label)?;
						},
						1 => {
							write!(out, "\
								@SP
								M=M-1
								A=M
								D=M+1
								@{}
								D=D+M
								@SP
								A=M
								A=M
								A=D-A
								M=D-A
							", label)?;
						},
						_ => { 
							write!(out, "\
								@SP
								M=M-1
								A=M
								D=M+1
								@{}
								D=D+M
								@{}
								D=D+A
								@SP
								A=M
								A=M
								A=D-A
								M=D-A
							", index, label)?;
						},
					}
				},
			};
			Ok(())
		}
	
		fn write_label_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				({}.{}${})
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_if_goto_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				AM=M-1
				D=M
				@{}.{}${}
				D;JNE
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_goto_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				@{}.{}${}
				0;JMP
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_return_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@{}
				0;JMP
			", RETURN_IMPL_LABEL)?;
			Ok(())
		}
	
		fn write_add_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				AM=M-1
				D=M
				A=A-1
				M=D+M
			")?;
			Ok(())
		}
	
		fn write_sub_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				AM=M-1
				D=M
				A=A-1
				M=M-D
			")?;
			Ok(())
		}
	
		fn write_neg_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				A=M-1
				M=-M
			")?;
			Ok(())
		}
	
		fn write_and_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				AM=M-1
				D=M
				A=A-1
				M=D&M
			")?;
			Ok(())
		}
	
		fn write_or_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				AM=M-1
				D=M
				A=A-1
				M=D|M
			")?;
			Ok(())
		}
	
		fn write_not_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP
				A=M-1
				M=!M
			")?;
			Ok(())
		}
	
		fn write_eq_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@RET_ADDRESS_EQ{}
				D=A
				@{}
				0;JMP
				(RET_ADDRESS_EQ{})
			", count, EQ_IMPL_LABEL, count)?;
			Ok(())
		}
	
		fn write_lt_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@RET_ADDRESS_LT{}
				D=A
				@{}
				0;JMP
				(RET_ADDRESS_LT{})
			", count, LT_IMPL_LABEL, count)?;
			Ok(())
		}
	
		fn write_gt_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@RET_ADDRESS_GT{}
				D=A
				@{}
				0;JMP
				(RET_ADDRESS_GT{})
			", count, GT_IMPL_LABEL, count)?;
			Ok(())
		}

		fn compose_segment_label(ctx: &InsContext, segment: VmSeg, index: u16) -> Result<CompactString, CodeError> {
			match segment {
				VmSeg::Constant => Ok(CompactString::new("")),
				VmSeg::Argument => Ok(CompactString::new("ARG")),
				VmSeg::Local => Ok(CompactString::new("LCL")),
				VmSeg::This => Ok(CompactString::new("THIS")),
				VmSeg::That => Ok(CompactString::new("THAT")),
				VmSeg::Pointer if index == 0 => Ok(CompactString::new("THIS")),
				VmSeg::Pointer if index == 1 => Ok(CompactString::new("THAT")),
				VmSeg::Pointer => return Err(CodeError::IndexOutOfBounds{segment, index, bounds: 0..1}),
				VmSeg::Temp => {
					match index {
						0 => Ok(CompactString::new("R5")),
						1 => Ok(CompactString::new("R6")),
						2 => Ok(CompactString::new("R7")),
						3 => Ok(CompactString::new("R8")),
						4 => Ok(CompactString::new("R9")),
						5 => Ok(CompactString::new("R10")),
						6 => Ok(CompactString::new("R11")),
						7 => Ok(CompactString::new("R12")),
						_ => Err(CodeError::IndexOutOfBounds{segment, index, bounds: 0..7}),
					}
				},
				VmSeg::Static => {
					if index as usize >= MAX_STATIC_VARIABLES {
						return Err(CodeError::IndexOutOfBounds{segment: VmSeg::Static, index, bounds: 0..(MAX_STATIC_VARIABLES - 1)});
					}
					let mut label = CompactString::from(&ctx.vm_file_name);
					label.push('.');
					let mut buf = ['\0'; 3];
					let mut i = 3;
					let mut num = index;
					while num > 0 {
						debug_assert!(i > 0);
						let digit = (num % 10) as u8;
						buf[i] = char::from_digit(digit.into(), 10).unwrap();
						num /= 10;
						i -= 1;
					}
					for c in buf {
						if c == '\0' {
							continue;
						}
						label.push(c);
					}
					Ok(label)
				},
			}
		}
	}
}
