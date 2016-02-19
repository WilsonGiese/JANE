use memory::{ Memory, ReadOnlyMemory, ReadWriteMemory };
use rom::Rom;
use std::fmt;
use std::boxed::*;

const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC; // Location of first instruction in memory
const IRQ_VECTOR: u16 = 0xFFFE;

/// CPU Flags
#[derive(Debug, Default)]
pub struct Flags {
	carry: bool,        // True if addition or shift carried, or subtraction didn't borrow
	zero: bool, 	    // True if Last operation result is 0
	irq_disable: bool,  // True if we want to inhibit IRQ interrupts (NMI allowed)
	decimal: bool,      // True if decimal mode
	overflow: bool,     // True if last ADC or SBC resulted in signed overflow
	negative: bool      // True if set bit 7 of last operation
}

/// CPU Registers
#[derive(Debug, Default)]
struct Registers {
	a: u8,    // Accumulator register used by the ALU
	x: u8,    // Indexing register
	y: u8,    // ''
	s: u8,    // Stack pointer
	pc: u16,  // Program counter (2 bytes wide)
	p: Flags, // Status register used by various instructions and the ALU
}

/// Model for the 6502 Microprocessor
pub struct CPU {
	registers: Registers,
	ram: ReadWriteMemory,
	cartridge: Box<Memory>
}

impl CPU {

	pub fn new(cartridge: Box<Memory>) -> CPU {

		CPU {
			registers: Registers::default(),
			ram: ReadWriteMemory::new(0x800),
			cartridge: cartridge
		}
	}

	/// Emulate CPU power up
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#At_power-up
	pub fn power_up(&mut self) {
		self.registers.a = 0;
		self.registers.x = 0;
		self.registers.y = 0;
		self.registers.s = 0xFD;
		self.registers.p.irq_disable = true;
		self.registers.pc = self.cartridge.loadw(RESET_VECTOR);
	}

	/// Emulate CPU reset
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#After_reset
	pub fn reset(&mut self) {
		// TODO: Reset state
	}

	pub fn run(&mut self) {
		println!("Running!");

		loop {
			// Get instruction from prg
			let instruction = self.load_pc();
			self.execute(instruction);
		}
	}

	fn load_pc(&mut self) -> u8 {
		let value = self.cartridge.load(self.registers.pc);
		self.registers.pc += 1;
		value
	}

	fn loadw_pc(&mut self) -> u16 {
		self.load_pc() as u16 | (self.load_pc() as u16) << 8
	}

	pub fn execute(&mut self, instruction: u8) {
		println!("Executing instruction: {:#X}", instruction);
		match instruction {
			0x78 => self.sei(),
			0xD8 => self.cld(),
			0xA9 => self.lda(),
			0xBD => println!("LDA Absolute, X"),
			0xAD => self.lda_a(),
			_ => panic!("Unsupported instruction: {:#X}", instruction)
		}
	}

	/// SEI Set interrupt disable status
	fn sei(&mut self) {
		self.registers.p.irq_disable = true;
	}

	/// CLD Clear decimal status
	fn cld(&mut self) {
		self.registers.p.decimal = false;
	}

	// LDA Load accumulator with memory
	fn lda(&mut self) {
		// TODO: Implement addressing modes
		self.registers.a = self.load_pc();
	}

	fn lda_a(&mut self) {
		let address = self.loadw_pc();
		self.registers.a = self.load(address);
	}
}

impl fmt::Display for CPU {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "CPU {{").unwrap();
		writeln!(f, "    A:  {:#X}", self.registers.a).unwrap();
		writeln!(f, "    X:  {:#X}", self.registers.x).unwrap();
		writeln!(f, "    Y:  {:#X}", self.registers.y).unwrap();
		writeln!(f, "    S:  {:#X}", self.registers.s).unwrap();
		writeln!(f, "    PC: {:#X}", self.registers.pc).unwrap();
		writeln!(f, "    P:  {:#?}", self.registers.p).unwrap();
		writeln!(f, "}}")
	}
}

//
// CPU Memory (2 Byte addressing)
//
//   0x0000 -> 0x7FFF : 2KB RAM
//   0x0800 -> 0x1FFF : Mirrored sections of RAM
//   0x2000 -> 0x2007 : PPU Registers
//   0x2008 -> 0x3FFF : Mirrored sections of PPU Registers
//   0x4000 -> 0x401F : APU and IO Registers
//   0x4020 -> 0xFFFF : Cartirdge Space
//

impl Memory for CPU {
	fn load(&self, address: u16) -> u8 {
		println!("CPU Load: {:#X}", address);
		match address {
			0x0000 ... 0x07FF => self.ram.load(address),
			0x0800 ... 0x0FFF => self.ram.load(address - 0x0800),
			0x1000 ... 0x17FF => self.ram.load(address - 0x1000),
			0x1800 ... 0x1FFF => self.ram.load(address - 0x1800),
			0x0800 ... 0x1FFF => unimplemented!(),
			0x2000 ... 0x2007 => unimplemented!(),
			0x2008 ... 0x3FFF => unimplemented!(),
			0x4000 ... 0x401F => unimplemented!(),
			0x4020 ... 0xFFFF => {
				println!("Accessing Cartridge");
				return self.cartridge.load(address);
			},
			_ => unreachable!()
		}
	}

	fn store(&mut self, address: u16, value: u8) {
		unimplemented!();
	}
}
