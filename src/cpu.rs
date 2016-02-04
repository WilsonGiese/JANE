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
	reg_p: u8, // TODO: Represent bit flags in a nice way
	reg_s: u8,
	reg_pc: u16
}

impl CPU {

	/// Emulate CPU power up
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#At_power-up
	pub fn power_up(&mut self) {
		self.reg_a = 0;
		self.reg_x = 0;
		self.reg_y = 0;
		self.reg_s = 0xFD;
		self.reg_p = 0x34;
	}

	/// Emulate CPU reset
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#After_reset
	pub fn reset(&mut self) {
		// TODO: Reset state
	}
}

impl fmt::Display for CPU {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		writeln!(f, "CPU {{").unwrap();
		writeln!(f, "\tA:  {:#X}", self.reg_a).unwrap();
		writeln!(f, "\tX:  {:#X}", self.reg_x).unwrap();
		writeln!(f, "\tY:  {:#X}", self.reg_y).unwrap();
		writeln!(f, "\tP:  {:#X}", self.reg_p).unwrap();
		writeln!(f, "\tS:  {:#X}", self.reg_s).unwrap();
		writeln!(f, "\tPC: {:#X}", self.reg_pc).unwrap();
		writeln!(f, "}}")
	}
}
