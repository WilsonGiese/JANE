use memory::{ Memory, ReadOnlyMemory, ReadWriteMemory };
use rom::Rom;
use std::fmt;

//
// CPU
//

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
	ram: ReadWriteMemory
}

impl CPU {

	pub fn new() -> CPU {
		CPU {
			registers: Registers::default(),
			ram: ReadWriteMemory::new(0x800),

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
	}

	/// Emulate CPU reset
	///		http://wiki.nesdev.com/w/index.php/CPU_power_up_state#After_reset
	pub fn reset(&mut self) {
		// TODO: Reset state
	}

	pub fn run(&mut self, rom: Rom) {
		println!("Running game!")
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
		self.registers.p.irq_disable = true;
	}

	/// CLD Clear decimal status
	fn cld(&mut self) {
		self.registers.p.decimal = false;
	}

	// LDA Load accumulator with memory
	fn lda(&mut self) {
		// TODO: Implement addressing modes
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

trait MappedMemory {
	fn load_from_prg(&self, address: u16) -> u8;
	fn load_from_chr(&self, address: u16) -> u8;
	fn store_to_prg(&mut self, address: u16, value: u8);
	fn store_to_chr(&mut self, address: u16, value: u8);

}

// NROM (0x0) Mapper for cartridge space
pub struct NRom {
	// TODO PRG RAM
	mirroring_prg: bool,
	prg: ReadOnlyMemory,
	chr: ReadOnlyMemory
}

impl NRom {
	fn new(prg_rom_units: usize, chr_rom_units: usize) -> NRom {
		NRom {
			mirroring_prg: prg_rom_units > 1,
			prg: ReadOnlyMemory::new(prg_rom_units * PRG_ROM_UNIT_SIZE),
			chr: ReadOnlyMemory::new(chr_rom_units * CHR_ROM_UNIT_SIZE)
		}
	}
}

// NROM Memory Map
// 0x6000 -> 0x7FFF: PRG RAM,
// 0x8000 -> 0xBFFF: First 16 KB of ROM.
// 0xC000 -> 0xFFFF: Last 16 KB of ROM (or mirror of first 16 KB)
impl MappedMemory for NRom {
	fn load_from_prg(&self, address: u16) -> u8 {
		if address < 0x8000 {
			unimplemented!()
		} else {
			// Mirror last section of PRG ROM if it is only 16 KB
			if self.mirroring_prg && address > 0xBFFF {
				return self.prg.load(address - 0xC000)
			}
			self.prg.load(address - 0x8000)
		}
	}

	fn load_from_chr(&self, address: u16) -> u8 { self.chr.load(address) }
	fn store_to_prg(&mut self, address: u16, value: u8) { self.prg.store(address, value) }
	fn store_to_chr(&mut self, address: u16, value: u8) { self.chr.store(address, value) }
}

///
/// Constants
///

/// PRG ROM Unit Size (16 KB)
const PRG_ROM_UNIT_SIZE: usize = 16 * 1024;

// CHR ROM Unit Size (8 KB)
const CHR_ROM_UNIT_SIZE: usize = 8 * 1024;
