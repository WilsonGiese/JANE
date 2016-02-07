use ines::Ines;

use std::fmt;


/// Model for the 6502 Microprocessor
/// The CPU consists of six registers:
///		A  - Accumulator register used by the ALU
/// 	X  - Indexing register
///		Y  - ''
///		P  - Status register used by various instructions and the ALU
///		S  - Stack pointer
///		PC - Program counter (2 bytes wide)
///
#[derive(Default, Debug)]
pub struct CPU {
	reg_a: u8,
	reg_x: u8,
	reg_y: u8,
	reg_s: u8,
	reg_pc: u16,
	reg_p: Flags,
}

impl CPU {

	/// Emulate CPU power up
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#At_power-up
	pub fn power_up(&mut self) {
		self.reg_a = 0;
		self.reg_x = 0;
		self.reg_y = 0;
		self.reg_s = 0xFD;
		self.reg_p.irq_disable = true;
	}

	/// Emulate CPU reset
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#After_reset
	pub fn reset(&mut self) {
		// TODO: Reset state
	}

	pub fn run(&mut self, game: Ines) {
		println!("Running game!");

		// Execute first instruction
		for i in 0..game.prg_rom.len() {
			self.execute(*game.prg_rom.get(i).unwrap());
		}
	}

	pub fn execute(&mut self, instruction: u8) {
		match instruction {
			0x78 => self.sei(),
			0xD8 => self.cld(),
			_ => panic!("Unsupported instruction: {:#X}", instruction)
		}
		println!("Executed instruction: {:#X}", instruction);
	}

	/// SEI Set interrupt disable status
	fn sei(&mut self) {
		self.reg_p.irq_disable = true;
	}

	/// CLD Clear decimal status
	fn cld(&mut self) {
		self.reg_p.decimal = false;
	}

	// LDA Load accumulator with memory
	fn lda(&mut self) {
		// TODO: Implement addressing modes
	}
}

impl fmt::Display for CPU {


	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "CPU {{").unwrap();
		writeln!(f, "    A:  {:#X}", self.reg_a).unwrap();
		writeln!(f, "    X:  {:#X}", self.reg_x).unwrap();
		writeln!(f, "    Y:  {:#X}", self.reg_y).unwrap();
		writeln!(f, "    S:  {:#X}", self.reg_s).unwrap();
		writeln!(f, "    PC: {:#X}", self.reg_pc).unwrap();
		writeln!(f, "    P:  {:#?}", self.reg_p).unwrap();
		writeln!(f, "}}")
	}
}

/// CPU Flags
#[derive(Default, Debug)]
pub struct Flags {
	carry: bool,        // True if addition or shift carried, or subtraction didn't borrow
	zero: bool, 	    // True if Last operation result is 0
	irq_disable: bool,  // True if we want to inhibit IRQ interrupts (NMI allowed)
	decimal: bool,      //
	overflow: bool,     // True if last ADC or SBC resulted in signed overflow
	negative: bool      // True if set bit 7 of last operation
}
