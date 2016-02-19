use memory::*;
use rom::*;

/// NROM (0x0) Mapper for PRG
pub struct NRomPRG {
	// TODO PRG RAM
	header: Header,
	is_mirroring_prg: bool,
	prg: ReadOnlyMemory
}

impl NRomPRG {
	pub fn new(header: Header, prg: Box<Vec<u8>>) -> NRomPRG {
		let is_mirroring_prg = header.prg_rom_size == 1;
		NRomPRG {
			header: header,
			prg: ReadOnlyMemory::new(prg),
			is_mirroring_prg: is_mirroring_prg,
		}
	}
}

// NROM Memory Map
// 0x6000 -> 0x7FFF: PRG RAM,
// 0x8000 -> 0xBFFF: First 16 KB of ROM.
// 0xC000 -> 0xFFFF: Last 16 KB of ROM (or mirror of first 16 KB)
impl Memory for NRomPRG {
	fn load(&self, address: u16) -> u8 {
		match address {
			0x8000u16 ... 0xFFFF => {
				if self.is_mirroring_prg && address > 0xBFFF {
					self.prg.load(address - 0xC000)
				} else {
					self.prg.load(address - 0x8000)
				}
			},
			_ => panic!("Invalid PRG memory access: {:#X}", address)
		}
	}

	fn store(&mut self, address: u16, value: u8) { self.prg.store(address, value); }
}

/// NROM (0x0) Mapper for CHR
pub struct NRomCHR {
	prg: ReadOnlyMemory
}
