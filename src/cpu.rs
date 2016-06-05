use memory::{ Memory, ReadWriteMemory };
use std::fmt;

const NMI_VECTOR: u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC; // Location of first instruction in memory
const IRQ_VECTOR: u16 = 0xFFFE;

/// CPU Flags
const CARRY_FLAG:    u8 = 1;      // Set if addition or shift carried, or subtraction didn't borrow
const ZERO_FLAG:     u8 = 1 << 1; // Set if Last operation result is 0
const IRQ_FLAG:      u8 = 1 << 2; // Set if we want to inhibit IRQ interrupts (NMI allowed)
const DECIMAL_FLAG:  u8 = 1 << 3; // Set if decimal mode
const OVERFLOW_FLAG: u8 = 1 << 6; // Set if last ADC or SBC resulted in signed overflow
const NEGATIVE_FLAG: u8 = 1 << 7; // Set if set bit 7 of last operation

// Set if BRK occured. The docs on nesdev.com are a bit confusing for this, says this bit has no
// effect, but also says "The only way for an IRQ handler to distinguish IRQ from BRK is to read the
// flags byte from the stack and test bit 4"
// We'll keep this flag for now and set it in the BRK instruction before pushing to the stack
const BREAK_FLAG:    u8 = 1 << 4;

/// CPU Registers
#[derive(Debug, Default)]
struct Registers {
	a: u8,    // Accumulator register used by the ALU
	x: u8,    // Indexing register
	y: u8,    // ''
	s: u8,    // Stack pointer
	pc: u16,  // Program counter (2 bytes wide)
	status: u8 // Status register used by various instructions and the ALU
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
		self.registers.pc = self.cartridge.loadw(RESET_VECTOR);
		self.set_status(IRQ_FLAG, true);
	}

	pub fn power_up_with_pc_override(&mut self, pc: u16) {
		self.registers.a = 0;
		self.registers.x = 0;
		self.registers.y = 0;
		self.registers.s = 0xFD;
		self.registers.pc = pc;
		self.set_status(IRQ_FLAG, true);
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

	// Status register operations

	fn set_status(&mut self, flag: u8, value: bool) {
		if value {
			self.registers.status |= flag;
		} else {
			self.registers.status &= !flag;
		}
	}

	fn get_status(&self, flag: u8) -> bool {
		self.registers.status & flag != 0
	}


	// Program Counter operations

	fn load_pc(&mut self) -> u8 {
		let value = self.cartridge.load(self.registers.pc);
		self.registers.pc += 1;
		value
	}

	fn get_pc(&mut self) -> u16 {
		let address = self.registers.pc;
		self.registers.pc += 1;
		address
	}

	fn loadw_pc(&mut self) -> u16 {
		self.load_pc() as u16 | (self.load_pc() as u16) << 8
	}

	fn getw_pc(&mut self) -> u16 {
		self.get_pc() as u16 | (self.get_pc() as u16) << 8
	}


	// Stack operations
	// Notes: Uses a descending stack (grows downwards)
	//        Hardcoded to Page 1 (0x100-0x1FF)

	// Push a value to the stack
	fn push(&mut self, value: u8) {
		let address = (self.registers.s as u16) | 0x100;
		self.store(address, value);
		self.registers.s -= 1;
	}

	// Push a word onto the stack
	fn pushw(&mut self, value: u16) {
		self.push((value >> 8) as u8);
		self.push(value as u8);
	}

	// Pop a value from the stack
	fn pop(&mut self) -> u8 {
		let address = (self.registers.s as u16) | 0x100;
		self.registers.s += 1;
		self.load(address)
	}


	// Addressing modes

	fn immediate_mode(&mut self) -> u16 {
		self.get_pc()
	}

	fn absolute_mode(&mut self) -> u16 {
		self.getw_pc()
	}

	fn absolute_x_mode(&mut self) -> u16 {
		self.getw_pc() + self.registers.x as u16
	}

	fn absolute_y_mode(&mut self) -> u16 {
		self.getw_pc() + self.registers.y as u16
	}

	fn zero_page_mode(&mut self) -> u16 {
		self.load_pc() as u16
	}

	fn zero_page_x_mode(&mut self) -> u16 {
		(self.load_pc() + self.registers.x) as u16
	}

	fn zero_page_y_mode(&mut self) -> u16 {
		(self.load_pc() + self.registers.y) as u16
	}

	fn indirect_x_mode(&mut self) -> u16 {
		let address = self.load_pc() + self.registers.x; // Zero page address
		self.loadw(address as u16) // Indirect address
	}

	fn indirect_y_mode(&mut self) -> u16 {
		let address = self.load_pc(); // Zero page address
		self.loadw(address as u16) + self.registers.y as u16 // Indirect address
	}

	fn relative_mode(&mut self) -> u16 {
		let address = self.load_pc();
		if address & 0x80 == 0x80 {
			self.registers.pc - (address & 0x7F) as u16
		} else {
			self.registers.pc + address as u16
		}
	}

	fn set_zn(&mut self, value: u8) {
		self.set_status(ZERO_FLAG, value == 0);
		self.set_status(NEGATIVE_FLAG, value & 0x80 != 0);
	}

	pub fn execute(&mut self, instruction: u8) {
		println!("Executing instruction: {:#X}", instruction);
		match instruction {

			// ADC
			0x69 => { let address = self.immediate_mode(); self.adc(address); },
			0x65 => { let address = self.zero_page_mode(); self.adc(address); },
			0x75 => { let address = self.zero_page_x_mode(); self.adc(address); },
			0x60 => { let address = self.absolute_mode(); self.adc(address); },
			0x7D => { let address = self.absolute_x_mode(); self.adc(address); },
			0x79 => { let address = self.absolute_y_mode(); self.adc(address); },
			0x61 => { let address = self.indirect_x_mode(); self.adc(address); },
			0x71 => { let address = self.indirect_y_mode(); self.adc(address); },

			// AND
			0x29 => { let address = self.immediate_mode(); self.and(address); },
			0x25 => { let address = self.zero_page_mode(); self.and(address); },
			0x35 => { let address = self.zero_page_x_mode(); self.and(address); },
			0x2D => { let address = self.absolute_mode(); self.and(address); },
			0x3D => { let address = self.absolute_x_mode(); self.and(address); },
			0x39 => { let address = self.absolute_y_mode(); self.and(address); },
			0x21 => { let address = self.indirect_x_mode(); self.and(address); },
			0x31 => { let address = self.indirect_y_mode(); self.and(address); },

			// ASL
			0x06 => { let address = self.zero_page_mode(); self.asl(address); },
			0x16 => { let address = self.zero_page_x_mode(); self.asl(address); },
			0x0E => { let address = self.absolute_mode(); self.asl(address); },
			0x1E => { let address = self.absolute_x_mode(); self.asl(address); },
			0x0A => self.asla(),

			// BRANCH Instructions
			0x90 => self.bcc(),
			0xb0 => self.bcs(),
			0xF0 => self.beq(),
			0x30 => self.bmi(),
			0xD0 => self.bne(),
			0x10 => self.bpl(),
			0x50 => self.bvc(),
			0x70 => self.bvs(),

			// BIT
			0x24 => { let address = self.zero_page_mode(); self.bit(address); },
			0x2C => { let address = self.absolute_mode(); self.bit(address); },

			// DECREMENT Instructions
			0xC6 => { let address = self.zero_page_mode(); self.dec(address); },
			0xD6 => { let address = self.zero_page_x_mode(); self.dec(address); },
			0xCE => { let address = self.absolute_mode(); self.dec(address); },
			0xDE => { let address = self.absolute_x_mode(); self.dec(address); },
			0xCA => self.dex(),
			0x88 => self.dey(),

			// BREAK
			0x00 => self.brk(),

			// CMP
			0xC9 => { let address = self.immediate_mode(); self.cmp(address); },
			0xC5 => { let address = self.zero_page_mode(); self.cmp(address); },
			0xD5 => { let address = self.zero_page_x_mode(); self.cmp(address); },
			0xCD => { let address = self.absolute_mode(); self.cmp(address); },
			0xDD => { let address = self.absolute_x_mode(); self.cmp(address); },
			0xD9 => { let address = self.absolute_y_mode(); self.cmp(address); },
			0xC1 => { let address = self.indirect_x_mode(); self.cmp(address); },
			0xD1 => { let address = self.indirect_y_mode(); self.cmp(address); },

			// INCREMENT Instructions
			0xE6 => { let address = self.zero_page_mode(); self.inc(address); },
			0xF6 => { let address = self.zero_page_x_mode(); self.inc(address); },
			0xEE => { let address = self.absolute_mode(); self.inc(address); },
			0xFE => { let address = self.absolute_x_mode(); self.inc(address); },
			0xE8 => self.inx(),
			0xC8 => self.iny(),

			// JUMP Instructions
			0x4C => self.jmpa(),
			0x6C => self.jmpi(),

			// LDA
			0xA9 => { let address = self.immediate_mode(); self.lda(address); },
			0xA5 => { let address = self.zero_page_mode(); self.lda(address); },
			0xB5 => { let address = self.zero_page_x_mode(); self.lda(address); },
			0xAD => { let address = self.absolute_mode(); self.lda(address); },
			0xBD => { let address = self.absolute_x_mode(); self.lda(address); },
			0xB9 => { let address = self.absolute_y_mode(); self.lda(address); },
			0xA1 => { let address = self.indirect_x_mode(); self.lda(address); },
			0xB1 => { let address = self.indirect_y_mode(); self.lda(address); }

			// LDX
			0xA2 => { let address = self.immediate_mode(); self.ldx(address); },
			0xA6 => { let address = self.zero_page_mode(); self.ldx(address); },
			0xB6 => { let address = self.zero_page_y_mode(); self.ldx(address); },
			0xAE => { let address = self.absolute_mode(); self.ldx(address); },
			0xBE => { let address = self.absolute_y_mode(); self.lda(address); }

			// LDY
			0xA0 => { let address = self.immediate_mode(); self.ldy(address); },
			0xA4 => { let address = self.zero_page_mode(); self.ldy(address); },
			0xB4 => { let address = self.zero_page_x_mode(); self.ldy(address); },
			0xAC => { let address = self.absolute_mode(); self.ldy(address); },
			0xBC => { let address = self.absolute_x_mode(); self.ldy(address); }

			0x78 => self.sei(),
			0xD8 => self.cld(),
			0x18 => self.clc(),
			0x58 => self.cli(),
			0xB8 => self.clv(),

			0xEA => self.nop(),
			_ => panic!("Unsupported instruction: {:#X}", instruction)
		}
	}

	// ADC - Add memory to accumulator with carry
	// A + M + C -> C, A
	fn adc(&mut self, address: u16) {
		let value = self.load(address);
		let mut new_a = self.registers.a as u16 + value as u16;

		if self.get_status(CARRY_FLAG) {
			new_a += 1;
		}
		self.set_status(CARRY_FLAG, new_a > 0xFF);

		if (self.registers.a ^ value) & 0x80 == 0 {
			if (self.registers.a ^ new_a as u8) & 0x80 == 0x80 {
				self.set_status(OVERFLOW_FLAG, true);
			}
		}
		self.set_zn(new_a as u8);
		self.registers.a = new_a as u8;
	}

	// AND - Apply bitwise AND to accumulator with memory
	// A & M -> A
	fn and(&mut self, address: u16) {
		let value = self.load(address);
		let new_a = self.registers.a & value;
		self.set_zn(new_a);
		self.registers.a = new_a;
	}

	// ASL - Shift memory left one bit
	// M << 1 -> M
	fn asl(&mut self, address: u16) {
		let mut value = self.load(address);
		self.set_status(CARRY_FLAG, value & 0x80 == 0x80);
		value <<= 1;
		self.set_zn(value);
		self.store(address, value);
	}

	// ASL - Shift accumulator left one bit
	// A << 1 -> A
	fn asla(&mut self) {
		let a = self.registers.a;
		self.set_status(CARRY_FLAG, a & 0x80 == 0x80);
		let new_a = self.registers.a << 1;
		self.set_zn(new_a);
		self.registers.a = new_a;
	}

	// BCC - Branch on carry clear
	// Branch on Carry == 0
	// Uses relative addressing mode; PC + value @ address
	fn bcc(&mut self) {
		if !self.get_status(CARRY_FLAG) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BCS - Branch on carry set
	// Branch on Carry == 1
	fn bcs(&mut self) {
		if self.get_status(CARRY_FLAG) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BEQ - Branch on Zero
	// Branch on Zero == 1
	fn beq(&mut self) {
		if self.get_status(ZERO_FLAG) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BIT - Test bits in memory with accumulator
	// A & M, M7 -> N, M6 -> V
	fn bit(&mut self, address: u16) {
		let value = self.load(address);
		self.set_status(NEGATIVE_FLAG, value & 0x80 == 0x80);
		self.set_status(OVERFLOW_FLAG, value & 0x40 == 0x40);
		let a = self.registers.a;
		self.set_status(ZERO_FLAG, a & value == 0);
	}

	// BMI - Branch on result minus
	// Branch on Negative == 1
	fn bmi(&mut self) {
		if self.get_status(NEGATIVE_FLAG) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BNE - Branch on result not zero
	// Branch on Zero == 0
	fn bne(&mut self) {
		if !self.get_status(ZERO_FLAG)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BPL - Branch on result plus
	// Branch on Negative == 0
	fn bpl(&mut self) {
		if !self.get_status(NEGATIVE_FLAG)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BRK - Fork break
	// Forced Interrupt PC + 2 toS P toS
	fn brk(&mut self) {
		let pc = self.registers.pc;
		self.pushw(pc + 1);
		self.set_status(BREAK_FLAG, true);
		let sr = self.registers.status;
		self.push(sr);
		self.set_status(IRQ_FLAG, true);
		self.registers.pc = self.loadw(IRQ_VECTOR);
	}

	// BVC - Branch on overflow clear
	// Branch on Overflow == 0
	fn bvc(&mut self) {
		if !self.get_status(OVERFLOW_FLAG)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BVS - Branch on overflow set
	// Branch on Overflow == 1
	fn bvs(&mut self) {
		if self.get_status(OVERFLOW_FLAG)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// DEC - Decrement memory by one
	// M - 1 -> M
	fn dec(&mut self, address: u16) {
		let value = self.load(address);
		self.store(address, value - 1);
	}

	// DEX - Decrement X by one
	// X - 1 -> X
	fn dex(&mut self) {
		self.registers.x -= 1;
	}

	// DEY - Decrement Y by one
	// Y - 1 -> Y
	fn dey(&mut self) {
		self.registers.y -= 1;
	}

	// INC - Increment memory by one
	// M + 1 -> M
	fn inc(&mut self, address: u16) {
		let value = self.load(address);
		self.store(address, value + 1);
	}

	// INX - Increment X by one
	// X + 1 -> X
	fn inx(&mut self) {
		self.registers.x += 1;
	}

	// INY - Increment Y by one
	// Y + 1 -> Y
	fn iny(&mut self) {
		self.registers.y += 1;
	}

	// JMP - Load PC in absolute mode
	// Absolute mode for this instruction; instead of loading the value at PC + 1, PC + 2, we jump
	// to it by setting PC.
	// (PC + 1) -> PC Low
	// (PC + 2) -> PC High
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

	// LDA - Load Accumulator with memory
	// Operation: M -> A
	fn lda(&mut self, address: u16) {
		self.registers.a = self.load(address);
	}

	// LDX - Load X with memory
	// Operation: M -> X
	fn ldx(&mut self, address: u16) {
		self.registers.x = self.load(address);
	}

	// LDY - Load Y with memory
	// Operation M -> Y
	fn ldy(&mut self, address: u16) {
		self.registers.y = self.load(address);
	}

	/// SEI - Set interrupt disable status
	fn sei(&mut self) {
		self.set_status(IRQ_FLAG, true);
	}

	/// CLD - Clear decimal status
	fn cld(&mut self) {
		self.set_status(DECIMAL_FLAG, false);
	}

	/// CLC - Clear carry flag
	fn clc(&mut self) {
		self.set_status(CARRY_FLAG, false);
	}

	/// CLI - Clear interrupt disable flag
	fn cli(&mut self) {
		self.set_status(IRQ_FLAG, false);
	}

	/// CLV - Clear overflow flag
	fn clv(&mut self) {
		self.set_status(OVERFLOW_FLAG, false);
	}

	// CMP - Compare memory and accumulator
	// A - M
	fn cmp(&mut self, address: u16) {
		let value = self.load(address);
		let a = self.registers.a;
		self.set_status(CARRY_FLAG, a >= value);
		self.set_zn(a - value);
	}

	fn nop(&self) { }
}

impl fmt::Display for CPU {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "CPU {{").unwrap();
		writeln!(f, "    A:  {:#X}", self.registers.a).unwrap();
		writeln!(f, "    X:  {:#X}", self.registers.x).unwrap();
		writeln!(f, "    Y:  {:#X}", self.registers.y).unwrap();
		writeln!(f, "    S:  {:#X}", self.registers.s).unwrap();
		writeln!(f, "    PC: {:#X}", self.registers.pc).unwrap();
		writeln!(f, "    P:  {:#?}", self.registers.status).unwrap();
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
			0x0000 ... 0x1FFF => self.ram.load(address & 0x7FF),
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
		println!("CPU Store: {:#X} = {:#X}", address, value);
		match address {
			0x0000 ... 0x07FF => self.ram.store(address & 0x7FF, value),
			0x2000 ... 0x2007 => unimplemented!(),
			0x2008 ... 0x3FFF => unimplemented!(),
			0x4000 ... 0x401F => unimplemented!(),
			0x4020 ... 0xFFFF => {
				println!("Accessing Cartridge");
				return self.cartridge.store(address, value);
			},
			_ => unreachable!()
		}
	}
}
