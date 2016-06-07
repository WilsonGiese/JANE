use memory::{ Memory, ReadWriteMemory };
use std::fmt;

const NMI_VECTOR:   u16 = 0xFFFA;
const RESET_VECTOR: u16 = 0xFFFC; // Location of first instruction in memory
const IRQ_VECTOR:   u16 = 0xFFFE;

/// CPU Status Flags
enum Flag {
	/// Set if addition or shift carried, or subtraction didn't borrow
	Carry    = 1,
	/// Set if last operation result is 0
	Zero     = 1 << 1,
	/// Set if we want to inhibit IRQ interrupts (NMI allowed)
	Irq      = 1 << 2,
	/// Set if decimal mode
	Decimal  = 1 << 3,
	/// Set if break occured. The docs on nesdev.com are a bit confusing for this, says this bit
	/// has no effect, but also says "The only way for an IRQ handler to distinguish IRQ from BRK
	/// is to read the flags byte from the stack and test bit 4"
	Break    = 1 << 4,
	/// Set if last ADC or SBC resulted in signed overflow
	Overflow = 1 << 6,
	/// Set if set bit 7 of last operation
	Negative = 1 << 7,
}

/// CPU Registers
#[derive(Debug, Default)]
struct Registers {
	a: u8,     // Accumulator register used by the ALU
	x: u8,     // Indexing register
	y: u8,     // ''
	s: u8,     // Stack pointer
	pc: u16,   // Program counter (2 bytes wide)
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
		self.set_status(Flag::Irq, true);
	}

	pub fn power_up_with_pc_override(&mut self, pc: u16) {
		self.registers.a = 0;
		self.registers.x = 0;
		self.registers.y = 0;
		self.registers.s = 0xFD;
		self.registers.pc = pc;
		self.set_status(Flag::Irq, true);
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

	fn set_status(&mut self, flag: Flag, value: bool) {
		if value {
			self.registers.status |= flag as u8;
		} else {
			self.registers.status &= !(flag as u8);
		}
	}

	fn get_status(&self, flag: Flag) -> bool {
		self.registers.status & flag as u8 != 0
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

	// Pull a value from the stack
	fn pull(&mut self) -> u8 {
		let address = (self.registers.s as u16) | 0x100;
		self.registers.s += 1;
		self.load(address)
	}

	// Pull a word from the stack
	fn pullw(&mut self) -> u16 {
		let word = self.pull();
		(word | self.pull()) as u16
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
		self.set_status(Flag::Zero, value == 0);
		self.set_status(Flag::Negative, value & 0x80 != 0);
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

			// BREAK
			0x00 => self.brk(),

			// CLEAR Instructions
			0xD8 => self.cld(),
			0x18 => self.clc(),
			0x58 => self.cli(),
			0xB8 => self.clv(),

			// CMP
			0xC9 => { let address = self.immediate_mode(); self.cmp(address); },
			0xC5 => { let address = self.zero_page_mode(); self.cmp(address); },
			0xD5 => { let address = self.zero_page_x_mode(); self.cmp(address); },
			0xCD => { let address = self.absolute_mode(); self.cmp(address); },
			0xDD => { let address = self.absolute_x_mode(); self.cmp(address); },
			0xD9 => { let address = self.absolute_y_mode(); self.cmp(address); },
			0xC1 => { let address = self.indirect_x_mode(); self.cmp(address); },
			0xD1 => { let address = self.indirect_y_mode(); self.cmp(address); },

			// CPX
			0xE0 => { let address = self.immediate_mode(); self.cpx(address) },
			0xE4 => { let address = self.zero_page_mode(); self.cpx(address) },
			0xEC => { let address = self.absolute_mode(); self.cpx(address) },

			// CPY
			0xC0 => { let address = self.immediate_mode(); self.cpy(address) },
			0xC4 => { let address = self.zero_page_mode(); self.cpy(address) },
			0xCC => { let address = self.absolute_mode(); self.cpy(address) },

			// DECREMENT Instructions
			0xC6 => { let address = self.zero_page_mode(); self.dec(address); },
			0xD6 => { let address = self.zero_page_x_mode(); self.dec(address); },
			0xCE => { let address = self.absolute_mode(); self.dec(address); },
			0xDE => { let address = self.absolute_x_mode(); self.dec(address); },
			0xCA => self.dex(),
			0x88 => self.dey(),

			// EOR
			0x49 => { let address = self.immediate_mode(); self.eor(address); },
			0x45 => { let address = self.zero_page_mode(); self.eor(address); },
			0x55 => { let address = self.zero_page_x_mode(); self.eor(address); },
			0x4D => { let address = self.absolute_mode(); self.eor(address); },
			0x5D => { let address = self.absolute_x_mode(); self.eor(address); },
			0x59 => { let address = self.absolute_y_mode(); self.eor(address); },
			0x41 => { let address = self.indirect_x_mode(); self.eor(address); },
			0x51 => { let address = self.indirect_y_mode(); self.eor(address); },

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
			0x20 => self.jsr(),

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

			// LSR
			0x46 => { let address = self.zero_page_mode(); self.lsr(address); },
			0x56 => { let address = self.zero_page_x_mode(); self.lsr(address); },
			0x4E => { let address = self.absolute_mode(); self.lsr(address); },
			0x5E => { let address = self.absolute_x_mode(); self.lsr(address); },
			0x4A => self.lsra(),

			// NOP
			0xEA => self.nop(),

			// ORA
			0x09 => { let address = self.immediate_mode(); self.ora(address); },
			0x05 => { let address = self.zero_page_mode(); self.ora(address); },
			0x15 => { let address = self.zero_page_x_mode(); self.ora(address); },
			0x0D => { let address = self.absolute_mode(); self.ora(address); },
			0x1D => { let address = self.absolute_x_mode(); self.ora(address); },
			0x19 => { let address = self.absolute_y_mode(); self.ora(address); },
			0x01 => { let address = self.indirect_x_mode(); self.ora(address); },
			0x11 => { let address = self.indirect_y_mode(); self.ora(address); }

			// Push & Pull Instructions
			0x48 => self.pha(),
			0x08 => self.php(),
			0x68 => self.pla(),
			0x28 => self.plp(),

			// Return Instructions
			0x4D => self.rti(),
			0x60 => self.rts(),


			// SET Instructions
			0x38 => self.sec(),
			0xF8 => self.sed(),
			0x78 => self.sei(),

			_ => panic!("Unsupported instruction: {:#X}", instruction)
		}
	}

	// ADC - Add memory to accumulator with carry
	// A + M + C -> C, A
	fn adc(&mut self, address: u16) {
		let value = self.load(address);
		let mut new_a = self.registers.a as u16 + value as u16;

		if self.get_status(Flag::Carry) {
			new_a += 1;
		}
		self.set_status(Flag::Carry, new_a > 0xFF);

		if (self.registers.a ^ value) & 0x80 == 0 {
			if (self.registers.a ^ new_a as u8) & 0x80 == 0x80 {
				self.set_status(Flag::Overflow, true);
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
		self.set_status(Flag::Carry, value & 0x80 == 0x80);
		value <<= 1;
		self.set_zn(value);
		self.store(address, value);
	}

	// ASL - Shift accumulator left one bit
	// A << 1 -> A
	fn asla(&mut self) {
		let a = self.registers.a;
		self.set_status(Flag::Carry, a & 0x80 == 0x80);
		let new_a = self.registers.a << 1;
		self.set_zn(new_a);
		self.registers.a = new_a;
	}

	// BCC - Branch on carry clear
	// Branch on Carry == 0
	// Uses relative addressing mode; PC + value @ address
	fn bcc(&mut self) {
		if !self.get_status(Flag::Carry) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BCS - Branch on carry set
	// Branch on Carry == 1
	fn bcs(&mut self) {
		if self.get_status(Flag::Carry) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BEQ - Branch on Zero
	// Branch on Zero == 1
	fn beq(&mut self) {
		if self.get_status(Flag::Zero) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BIT - Test bits in memory with accumulator
	// A & M, M7 -> N, M6 -> V
	fn bit(&mut self, address: u16) {
		let value = self.load(address);
		self.set_status(Flag::Negative, value & 0x80 == 0x80);
		self.set_status(Flag::Overflow, value & 0x40 == 0x40);
		let a = self.registers.a;
		self.set_status(Flag::Zero, a & value == 0);
	}

	// BMI - Branch on result minus
	// Branch on Negative == 1
	fn bmi(&mut self) {
		if self.get_status(Flag::Negative) {
			self.registers.pc = self.relative_mode();
		}
	}

	// BNE - Branch on result not zero
	// Branch on Zero == 0
	fn bne(&mut self) {
		if !self.get_status(Flag::Zero)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BPL - Branch on result plus
	// Branch on Negative == 0
	fn bpl(&mut self) {
		if !self.get_status(Flag::Negative)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BRK - Fork break
	// Forced Interrupt PC + 2 toS P toS
	fn brk(&mut self) {
		let pc = self.registers.pc;
		self.pushw(pc + 1);
		self.set_status(Flag::Break, true);
		let sr = self.registers.status;
		self.push(sr);
		self.set_status(Flag::Irq, true);
		self.registers.pc = self.loadw(IRQ_VECTOR);
	}

	// BVC - Branch on overflow clear
	// Branch on Overflow == 0
	fn bvc(&mut self) {
		if !self.get_status(Flag::Overflow)  {
			self.registers.pc = self.relative_mode();
		}
	}

	// BVS - Branch on overflow set
	// Branch on Overflow == 1
	fn bvs(&mut self) {
		if self.get_status(Flag::Overflow)  {
			self.registers.pc = self.relative_mode();
		}
	}

	/// CLD - Clear decimal status
	fn cld(&mut self) {
		self.set_status(Flag::Decimal, false);
	}

	/// CLC - Clear carry flag
	fn clc(&mut self) {
		self.set_status(Flag::Carry, false);
	}

	/// CLI - Clear interrupt disable flag
	fn cli(&mut self) {
		self.set_status(Flag::Irq, false);
	}

	/// CLV - Clear overflow flag
	fn clv(&mut self) {
		self.set_status(Flag::Overflow, false);
	}

	// Compare Helper
	fn compare(&mut self, a: u8, b: u8) {
		self.set_status(Flag::Carry, a >= b);
		self.set_zn(a - b);
	}

	// CMP - Compare memory and accumulator
	// A - M
	fn cmp(&mut self, address: u16) {
		let a = self.registers.a;
		let b = self.load(address);
		self.compare(a, b);
	}

	// CPX - Compare memory and x
	// X - M
	fn cpx(&mut self, address: u16) {
		let a = self.registers.x;
		let b = self.load(address);
		self.compare(a, b);
	}

	// CPY - Compare memory and y
	// Y - M
	fn cpy(&mut self, address: u16) {
		let a = self.registers.y;
		let b = self.load(address);
		self.compare(a, b);
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

	// EOR - Exclusive OR memory with accumulator
	// A ^ M -> A
	fn eor(&mut self, address: u16) {
		let value = self.load(address);
	 	let new_a = self.registers.a ^ value;
		self.set_zn(new_a);
		self.registers.a = new_a;
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

	// JSR - Jump to new location saving return address
	// PC + 2 toS, (PC + 1) -> PCL
	//             (PC + 2) -> PCH
	fn jsr(&mut self) {
		let address = self.loadw_pc();
		let pc = self.registers.pc;
		self.pushw(pc - 1);
		self.registers.pc = address;
	}

	// LDA - Load Accumulator with memory
	// Operation: M -> A
	fn lda(&mut self, address: u16) {
		let new_a = self.load(address);
		self.registers.a = new_a;
		self.set_zn(new_a);
	}

	// LDX - Load X with memory
	// Operation: M -> X
	fn ldx(&mut self, address: u16) {
		let new_x = self.load(address);
		self.registers.x = new_x;
		self.set_zn(new_x);
	}

	// LDY - Load Y with memory
	// Operation M -> Y
	fn ldy(&mut self, address: u16) {
		let new_y = self.load(address);
		self.registers.y = new_y;
		self.set_zn(new_y);
	}

	// LSR - Shift memory right one bit
	// M >> 1 -> M
	fn lsr(&mut self, address: u16) {
		let value = self.load(address);
		self.store(address, value >> 1);
	}

	// LSR - Shift accumulator right one bit
	// A >> 1 -> A
	fn lsra(&mut self) {
		self.registers.a = self.registers.a >> 1;
	}

	// NOP - No Operation
	fn nop(&self) { }

	// ORA - OR memory with accumulator
	// A | M -> A
	fn ora(&mut self, address: u16) {
		let new_a = self.registers.a | self.load(address);;
		self.registers.a = new_a;
		self.set_zn(new_a);
	}

	// PHA - Push accumulator onto stack
	// A -> toS
	fn pha(&mut self) {
		let a = self.registers.a;
		self.push(a);
	}

	// PHP - Push processor status onto stack
	// P -> toS
	fn php(&mut self) {
		let p = self.registers.status;
		self.push(p);
	}

	// PLA - Pull accumulator from stack
	// toS -> A
	fn pla(&mut self) {
		self.registers.a = self.pull();
	}

	// PLP - Pull processor status from stack
	// toS -> P
	fn plp(&mut self) {
		self.registers.status = self.pull();
	}

	// ROL - Rotate memory one bit left

	// ROL - Rotate accumulator one bit left

	// ROR - Rotate memory one bit right

	// ROR - Rotate accumulator one bit right

	// RTI - Return from interrupt
	fn rti(&mut self) {
		self.registers.s = self.pull();
		self.registers.pc = self.pullw();
	}

	// RTS - Return from subroutine
	// toS -> PC, PC + 1 -> PC
	fn rts(&mut self) {
		self.registers.pc = self.pullw() + 1;
	}

	// SEC - Set carry
	// 1 -> Status(Carry)
	fn sec(&mut self) {
		self.set_status(Flag::Carry, true);
	}

	// SED = Set decimal mode
	// 1 -> Status(Decimal)
	fn sed(&mut self) {
		self.set_status(Flag::Decimal, true);
	}

	// SEI - Set interrupt disable status
	// 1 -> Status(IRQ)
	fn sei(&mut self) {
		self.set_status(Flag::Irq, true);
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
