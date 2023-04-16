use std::io::Write;
use compact_str::CompactString;
use crate::tokenizer::*;
use crate::parser::*;
use crate::errors::*;

const CALL_STACK_BASE_ADDRESS: u16 = 256;
const TEMP_SEGMENT_BASE_ADDRESS: u16 = 5;
const MAX_STATIC_VARIABLES: usize = 240;

const EQ_IMPL_LABEL: &'static str = "__EQ_IMPL";
const GT_IMPL_LABEL: &'static str = "__GT_IMPL";
const LT_IMPL_LABEL: &'static str = "__LT_IMPL";
const RETURN_IMPL_LABEL: &'static str = "__RETURN_IMPL";
const CALL_IMPL_LABEL: &'static str = "__CALL_IMPL";
const ENTRY_IMPL_LABEL: &'static str = "__ENTRY_IMPL";

pub struct Coder {
	call_count: usize,
	eq_count: usize,
	lt_count: usize,
	gt_count: usize,
}

pub struct InsContext {
	pub vm_file_name: CompactString,
	pub vm_function_name: CompactString,
}

impl InsContext {
	pub fn new() -> Self {
		InsContext{vm_file_name: CompactString::new(""), vm_function_name: CompactString::new("")}
	}
}

impl Coder {
	pub fn new() -> Self {
		Coder{call_count: 0, eq_count: 0, lt_count: 0, gt_count: 0}
	}

	pub fn write_core_impl<W: Write>(&mut self, out: &mut W) -> Result<(), CodeError> {
		let bootstrap_impl = format!("\
			@{}\n\
			D=A\n\
			@SP\n\
			M=D\n\
			@0\n\
			D=A\n\
			@R13\n\
			M=D\n\
			@sys.init\n\
			D=A\n\
			@R14\n\
			M=D\n\
			@__RET_SYS_INIT\n\
			D=A\n\
			@{}\n\
			0;JMP\n\
			(__RET_SYS_INIT)\n\
			(__HANG)\n\
			@__HANG\n\
			0;JMP\n\
		", CALL_STACK_BASE_ADDRESS, CALL_IMPL_LABEL);
		let eq_impl = format!("\
			({})\n\
			@R15\n\
			M=D\n\
			@SP\n\
			AM=M-1\n\
			D=M\n\
			A=A-1\n\
			D=M-D\n\
			M=0\n\
			@__END_EQ\n\
			D;JNE\n\
			@SP\n\
			A=M-1\n\
			M=-1\n\
			(__END_EQ)\n\
			@R15\n\
			A=M\n\
			0;JMP\n\
		", EQ_IMPL_LABEL);
		let gt_impl = format!("\
			({})\n\
			@R15\n\
			M=D\n\
			@SP\n\
			AM=M-1\n\
			D=M\n\
			A=A-1\n\
			D=M-D\n\
			M=0\n\
			@__END_GT\n\
			D;JLE\n\
			@SP\n\
			A=M-1\n\
			M=-1\n\
			(__END_GT)\n\
			@R15\n\
			A=M\n\
			0;JMP\n\
		", GT_IMPL_LABEL);
		let lt_impl = format!("\
			({})\n\
			@R15\n\
			M=D\n\
			@SP\n\
			AM=M-1\n\
			D=M\n\
			A=A-1\n\
			D=M-D\n\
			M=0\n\
			@__END_LT\n\
			D;JGE\n\
			@SP\n\
			A=M-1\n\
			M=-1\n\
			(__END_LT)\n\
			@R15\n\
			A=M\n\
			0;JMP\n\
		", LT_IMPL_LABEL);
		let return_impl = format!("\
			({})\n\
			@5\n\
			D=A\n\
			@LCL\n\
			A=M-D\n\
			D=M\n\
			@R13\n\
			M=D\n\
			@SP\n\
			AM=M-1\n\
			D=M\n\
			@ARG\n\
			A=M\n\
			M=D\n\
			D=A\n\
			@SP\n\
			M=D+1\n\
			@LCL\n\
			D=M\n\
			@R14\n\
			AM=D-1\n\
			D=M\n\
			@THAT\n\
			M=D\n\
			@R14\n\
			AM=M-1\n\
			D=M\n\
			@THIS\n\
			M=D\n\
			@R14\n\
			AM=M-1\n\
			D=M\n\
			@ARG\n\
			M=D\n\
			@R14\n\
			AM=M-1\n\
			D=M\n\
			@LCL\n\
			M=D\n\
			@R13\n\
			A=M\n\
			0;JMP\n\
		", RETURN_IMPL_LABEL);
		let call_impl = format!("\
			({})\n\
			@SP\n\
			A=M\n\
			M=D\n\
			@LCL\n\
			D=M\n\
			@SP\n\
			AM=M+1\n\
			M=D\n\
			@ARG\n\
			D=M\n\
			@SP\n\
			AM=M+1\n\
			M=D\n\
			@THIS\n\
			D=M\n\
			@SP\n\
			AM=M+1\n\
			M=D\n\
			@THAT\n\
			D=M\n\
			@SP\n\
			AM=M+1\n\
			M=D\n\
			@4\n\
			D=A\n\
			@R13\n\
			D=D+M\n\
			@SP\n\
			D=M-D\n\
			@ARG\n\
			M=D\n\
			@SP\n\
			MD=M+1\n\
			@LCL\n\
			M=D\n\
			@R14\n\
			A=M\n\
			0;JMP\n\
		", CALL_IMPL_LABEL);
	
		write!(out, "{}", bootstrap_impl)?;
		write!(out, "{}", eq_impl)?;
		write!(out, "{}", gt_impl)?;
		write!(out, "{}", lt_impl)?;
		write!(out, "{}", return_impl)?;
		write!(out, "{}", call_impl)?;
	
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
						({}.{})\n\
					", ctx.vm_file_name, name)?;
				},
				1 => {
					write!(out, "\
						({}.{})\n\
						@SP\n\
						AM=M+1\n\
						A=A-1\n\
						M=0\n\
					", ctx.vm_file_name, name)?;
				},
				2 => {
					write!(out, "\
						({}.{})\n\
						@SP\n\
						AM=M+1\n\
						A=A-1\n\
						M=0\n\
						@SP\n\
						AM=M+1\n\
						A=A-1\n\
						M=0\n\
					", ctx.vm_file_name, name)?;
				},
				_ => {
					write!(out, "\
						({}.{})\n\
						@{}\n\
						D=A\n\
						(__LOOP_{}.{})\n\
						D=D-1\n\
						@SP\n\
						AM=M+1\n\
						A=A-1\n\
						M=0\n\
						@__LOOP_{}.{}\n\
						D;JGT\n\
					", ctx.vm_file_name, name, locals_count, ctx.vm_file_name, name, ctx.vm_file_name, name)?;
				},
			};
			Ok(())
		}
	
		fn write_call_ins<W: Write>(out: &mut W, ctx: &InsContext, function: CompactString, args_count: u16, call_count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@{}\n\
				D=A\n\
				@R13\n\
				M=D\n\
				@{}.{}\n\
				D=A\n\
				@R14 \n\
				M=D\n\
				@{}.{}$ret.{}\n\
				D=A\n\
				@{}\n\
				0;JMP\n\
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
								@SP\n\
								M=M+1\n\
								A=M-1\n\
								M=0\n\
							")?;
						},
						1 => {
							write!(out, "\
								@SP\n\
								M=M+1\n\
								A=M-1\n\
								M=1\n\
							")?;
						},
						_ => { 
							write!(out, "\
								@{}\n\
								D=A\n\
								@SP\n\
								M=M+1\n\
								A=M-1\n\
								M=D\n\
							", index)?;
						},
					}
				},
				VmSeg::Static => {
					write!(out, "\
						@{}\n\
						D=M\n\
						@SP\n\
						AM=M+1\n\
						A=A-1\n\
						M=D\n\
					", label)?;
				},
				_ => {
					match index {
						0 => {
							write!(out, "\
								@{}\n\
								A=M\n\
								D=M\n\
								@SP\n\
								AM=M+1\n\
								A=A-1\n\
								M=D\n\
							", label)?;
						},
						1 => {
							write!(out, "\
								@{}\n\
								A=M+1\n\
								D=M\n\
								@SP\n\
								AM=M+1\n\
								A=A-1\n\
								M=D\n\
							", label)?;
						},
						_ => { 
							write!(out, "\
								@{}\n\
								D=A\n\
								@{}\n\
								A=M+D\n\
								D=M\n\
								@SP\n\
								AM=M+1\n\
								A=A-1\n\
								M=D\n\
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
						@SP\n\
						M=M-1\n\
						A=M\n\
						D=M\n\
						@{}\n\
						M=D\n\
					", label)?;
				},
				_ => {
					match index {
						0 => {
							write!(out, "\
								@SP\n\
								M=M-1\n\
								A=M\n\
								D=M\n\
								@{}\n\
								D=D+M\n\
								@SP\n\
								A=M\n\
								A=M\n\
								A=D-A\n\
								M=D-A\n\
							", label)?;
						},
						1 => {
							write!(out, "\
								@SP\n\
								M=M-1\n\
								A=M\n\
								D=M+1\n\
								@{}\n\
								D=D+M\n\
								@SP\n\
								A=M\n\
								A=M\n\
								A=D-A\n\
								M=D-A\n\
							", label)?;
						},
						_ => { 
							write!(out, "\
								@SP\n\
								M=M-1\n\
								A=M\n\
								D=M+1\n\
								@{}\n\
								D=D+M\n\
								@{}\n\
								D=D+A\n\
								@SP\n\
								A=M\n\
								A=M\n\
								A=D-A\n\
								M=D-A\n\
							", index, label)?;
						},
					}
				},
			};
			Ok(())
		}
	
		fn write_label_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				({}.{}${})\n\
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_if_goto_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				AM=M-1\n\
				D=M\n\
				@{}.{}${}\n\
				D;JNE\n\
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_goto_ins<W: Write>(out: &mut W, ctx: &InsContext, label: CompactString) -> Result<(), CodeError> {
			write!(out, "\
				@{}.{}${}\n\
				0;JMP\n\
			", ctx.vm_file_name, ctx.vm_function_name, label)?;
			Ok(())
		}
	
		fn write_return_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@{}\n\
				0;JMP\n\
			", RETURN_IMPL_LABEL)?;
			Ok(())
		}
	
		fn write_add_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				AM=M-1\n\
				D=M\n\
				A=A-1\n\
				M=D+M\n\
			")?;
			Ok(())
		}
	
		fn write_sub_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				AM=M-1\n\
				D=M\n\
				A=A-1\n\
				M=M-D\n\
			")?;
			Ok(())
		}
	
		fn write_neg_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				A=M-1\n\
				M=-M\n\
			")?;
			Ok(())
		}
	
		fn write_and_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				AM=M-1\n\
				D=M\n\
				A=A-1\n\
				M=D&M\n\
			")?;
			Ok(())
		}
	
		fn write_or_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				AM=M-1\n\
				D=M\n\
				A=A-1\n\
				M=D|M\n\
			")?;
			Ok(())
		}
	
		fn write_not_ins<W: Write>(out: &mut W) -> Result<(), CodeError> {
			write!(out, "\
				@SP\n\
				A=M-1\n\
				M=!M\n\
			")?;
			Ok(())
		}
	
		fn write_eq_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@__RET_EQ{}\n\
				D=A\n\
				@{}\n\
				0;JMP\n\
				(__RET_EQ{})\n\
			", count, EQ_IMPL_LABEL, count)?;
			Ok(())
		}
	
		fn write_lt_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@__RET_LT{}\n\
				D=A\n\
				@{}\n\
				0;JMP\n\
				(__RET_LT{})\n\
			", count, LT_IMPL_LABEL, count)?;
			Ok(())
		}
	
		fn write_gt_ins<W: Write>(out: &mut W, count: usize) -> Result<(), CodeError> {
			write!(out, "\
				@__RET_GT{}\n\
				D=A\n\
				@{}\n\
				0;JMP\n\
				(__RET_GT{})\n\
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
					let mut label = ctx.vm_file_name.clone();
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
