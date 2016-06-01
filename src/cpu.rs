use memory::{ Memory, ReadWriteMemory };
use std::fmt;

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

	pub fn power_up_with_pc_override(&mut self, pc: u16) {
		self.registers.a = 0;
		self.registers.x = 0;
		self.registers.y = 0;
		self.registers.s = 0xFD;
		self.registers.p.irq_disable = true;
		self.registers.pc = pc;
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

	fn immediate_mode(&mut self) -> u8 {
		self.load_pc()
	}

	fn absolute_mode(&mut self) -> u8 {
		let address = self.loadw_pc();
		self.load(address)
	}

	fn absolute_x_mode(&mut self) -> u8 {
		let address = self.loadw_pc() + self.registers.x as u16; // TODO handle overflow?
		self.load(address)
	}

	fn absolute_y_mode(&mut self) -> u8 {
		let address = self.loadw_pc() + self.registers.y as u16; // TODO handle overflow?
		self.load(address)
	}

	fn zero_page_mode(&mut self) -> u8 {
		let address = self.load_pc();
		self.load(address as u16)
	}

	fn zero_page_x_mode(&mut self) -> u8 {
		let address = self.load_pc() + self.registers.x;
		self.load(address as u16)
	}

	fn zero_page_y_mode(&mut self) -> u8 {
		let address = self.load_pc() + self.registers.y;
		self.load(address as u16)
	}

	fn inderect_x_mode(&mut self) -> u8 {
		let address = self.load_pc() + self.registers.x; // Zero page address
		let address = self.loadw(address as u16); // Indirect address
		self.load(address)
	}

	fn inderect_y_mode(&mut self) -> u8 {
		let address = self.load_pc(); // Zero page address
		let address = self.loadw(address as u16) + self.registers.y as u16; // Indirect address
		self.load(address)
	}

	pub fn execute(&mut self, instruction: u8) {
		println!("Executing instruction: {:#X}", instruction);
		match instruction {
			// LDA
			0xA9 => { let value = self.immediate_mode(); self.lda(value); },
			0xA5 => { let value = self.zero_page_mode(); self.lda(value); },
			0xB5 => { let value = self.zero_page_x_mode(); self.lda(value); },
			0xAD => { let value = self.absolute_mode(); self.lda(value); },
			0xBD => { let value = self.absolute_x_mode(); self.lda(value); },
			0xB9 => { let value = self.absolute_y_mode(); self.lda(value); },
			0xA1 => { let value = self.inderect_x_mode(); self.lda(value); },
			0xB1 => { let value = self.inderect_y_mode(); self.lda(value); }

			// LDX
			0xA2 => { let value = self.immediate_mode(); self.ldx(value); },
			0xA6 => { let value = self.zero_page_mode(); self.ldx(value); },
			0xB6 => { let value = self.zero_page_y_mode(); self.ldx(value); },
			0xAE => { let value = self.absolute_mode(); self.ldx(value); },
			0xBE => { let value = self.absolute_y_mode(); self.lda(value); }

			// LDY
			0xA0 => { let value = self.immediate_mode(); self.ldy(value); },
			0xA4 => { let value = self.zero_page_mode(); self.ldy(value); },
			0xB4 => { let value = self.zero_page_x_mode(); self.ldy(value); },
			0xAC => { let value = self.absolute_mode(); self.ldy(value); },
			0xBC => { let value = self.absolute_x_mode(); self.ldy(value); }

			// JUMP
			0x4C => { self.jmpa(); }
			0x6C => { self.jmpi(); }







			0x78 => self.sei(),
			0xD8 => self.cld(),
			0x18 => self.clc(),
			0x58 => self.cli(),
			0xB8 => self.clv(),
			_ => panic!("Unsupported instruction: {:#X}", instruction)
		}
	}

	// LDA - Load Accumulator with memory
	// Operation: M -> A
	fn lda(&mut self, value: u8) {
		self.registers.a = value;
	}

	// LDX - Load X with memory
	// Operation: M -> X
	fn ldx(&mut self, value: u8) {
		self.registers.x = value;
	}

	// LDY - Load Y with memory
	// Operation M -> Y
	fn ldy(&mut self, value: u8) {
		self.registers.y = value;
	}

	// JMP - Load PC in absolute mode
	// Absolute mode for this instruction; instead of loading the value at PC + 1, PC + 2, we jump
	// to it by setting PC.
	fn jmpa(&mut self) {
		let value = self.loadw_pc();
		println!("JMP {:#X}", value);
		self.registers.pc = value;
	}

	// JMP - Load PC in indirect mode
	// Indirect mode for this instruction; instead of loading the value at PC + 1, PC + 2, we take
	// load the word starting at PC + 1 and jump to it by setting PC
	fn jmpi(&mut self) {
		let address = self.loadw_pc();
		let value = self.loadw(address);
		println!("JMP {:#X}", value);
		self.registers.pc = value;
	}

	// DEC - Decrement memory by one
 	fn dec(&mut self, address: u16) {
 		let value = self.load(address);
 		self.store(address, value - 1);
 	}

 	// DEX - Decrement X by one
 	fn dex(&mut self) {
 		self.registers.x -= 1;
 	}

 	// DEY - Decrement X by one
 	fn dey(&mut self) {
 		self.registers.y -= 1;
 	}

	/// SEI - Set interrupt disable status
	fn sei(&mut self) {
		self.registers.p.irq_disable = true;
	}

	/// CLD - Clear decimal status
	fn cld(&mut self) {
		self.registers.p.decimal = false;
	}

	/// CLC - Clear carry flag
	fn clc(&mut self) {
		self.registers.p.carry = false;
	}

	/// CLI - Clear interrupt disable flag
	fn cli(&mut self) {
		self.registers.p.irq_disable = false;
	}

	/// CLV - Clear overflow flag
	fn clv(&mut self) {
		self.registers.p.overflow = false;
	}

	fn nop() { }
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
