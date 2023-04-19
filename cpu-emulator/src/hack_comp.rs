#![allow(dead_code)]


const SC_RAM_ADR: u16 = 0x4000;
const KB_RAM_ADR: u16 = 0x6001;

const A_REG: usize = 0;
const D_REG: usize = 1;
const P_REG: usize = 2;
const C_REG: usize = 3;

const CNREG: usize = 4;
const CNROM: usize = 0x8000;
const CNRAM: usize = 0x6001;

type Op = fn(&mut Box<[u16; CNRAM]>, &mut [u16; CNREG]);

fn op_null(_: &mut Box<[u16; CNRAM]>, _: &mut [u16; CNREG]) {}

fn op_dest_a(_: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[A_REG] = reg[C_REG];
}

fn op_dest_d(_: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[D_REG] = reg[C_REG];
}

fn op_dest_m(ram: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	ram[reg[A_REG] as usize] = reg[C_REG];
}

fn op_dest_ad(ram: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	op_dest_a(ram, reg);
	op_dest_d(ram, reg);
}

fn op_dest_am(ram: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	op_dest_a(ram, reg);
	op_dest_m(ram, reg);
}

fn op_dest_dm(ram: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	op_dest_d(ram, reg);
	op_dest_m(ram, reg);
}

fn op_dest_adm(ram: &mut Box<[u16; CNRAM]>, reg: &mut [u16; CNREG]) {
	op_dest_a(ram, reg);
	op_dest_d(ram, reg);
	op_dest_m(ram, reg);
}

static DEST_OP_TABLE: [Op; 8] = [
	op_null,
	op_dest_m,
	op_dest_d,
	op_dest_dm,
	op_dest_a,
	op_dest_am,
	op_dest_ad,
	op_dest_adm
];

fn op_comp_0(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = 0;
}

fn op_comp_1(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = 1;
}

fn op_comp_minus_1(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = -1;
}

fn op_comp_d(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = reg[D_REG];
}

fn op_comp_a(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = reg[A_REG];
}

fn op_comp_m(ram: &mut Box<[i16; CNRAM]>, reg: &mut [u16; CNREG]) {
	reg[C_REG] = ram[reg[A_REG] as usize];
}


pub struct HackComputer {
	rom: Box<[u16; CNROM]>,
	ram: Box<[i16; CNRAM]>,
	reg: [i32; CNREG],
}

impl HackComputer {
	pub fn new() -> Self {
		HackComputer{rom: Box::new([0; CNROM]), ram: Box::new([0; CNRAM]), reg: [0; CNREG]}
	}

	pub fn load(&mut self, program: &[u16]) -> Result<(), String> {
		if program.len() > CNROM {
			return Err("program too large".to_string());
		}
		self.rom.iter_mut().for_each(|m| *m = 0);
		for (i, m) in program.iter().enumerate() {
			self.rom[i] = *m;
		}
		Ok(())
	}

	pub fn reset() {
	}

	pub fn run() {
	}

	pub fn step(&mut self) {
		let ins = self.reg[P_REG];
		self.reg[P_REG] += 1;

		if ins & 0b1000000000000000 > 0 {
			let dest = (ins & 0b000_0_000000_111_000) >> 3;
			let comp = (ins & 0b000_1_111111_000_000) >> 6;
			let jump =  ins & 0b000_0_000000_000_111;

			DEST_OP_TABLE[dest as usize](&mut self.ram, &mut self.reg);
			COMP_OP_TABLE[dest as usize](&mut self.ram, &mut self.reg);
			JUMP_OP_TABLE[dest as usize](&mut self.ram, &mut self.reg);
		}
		else {
			self.reg[A_REG] = ins;
		}
	}
}
