use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::{Error, ErrorKind};
use std::io::Result;

/// Identifier should always be the first 4 bytes of iNES header
const IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

/// iNES Header (16 Bytes)
/// Format:
///   0-3: Identifier
///     4: PRG ROM size
///     5: CHR ROM size
///     6: Flags
///     7: Flags
///     8: PRG RAM size
///     9: Flags
///    10: Flags
/// 11-15: Zero filled
#[derive(Debug)]
pub struct Header {
	pub prg_rom_size: u8,
	pub chr_rom_size: u8,
	pub prg_ram_size: u8,
	pub flags6: Flags6,
	pub flags7: Flags7,
	pub mapper_number: u8,
	// TODO: Flags 9,10 (Ignoring for now; flags 9 is unused and flags 10 is unofficial)
}

impl Header {
	fn new(data: &[u8; 16]) -> Result<Header> {
		if data[0..4] != IDENTIFIER {
			Err(Error::new(ErrorKind::Other, "File is not in iNES file format!"))
		} else {
			let mut header = Header {
				prg_rom_size: data[4],
				chr_rom_size: data[5],
				prg_ram_size: data[8],
				flags6: Flags6::new(&data[6]),
				flags7: Flags7::new(&data[7]),
				mapper_number: 0
			};
			// Set mapper number by combing upper and lower bits from flags
			header.mapper_number = (header.flags7.mapper_upper << 4) & header.flags6.mapper_lower;
			Ok(header)
		}
	}
}

/// Flags 6 (1 Byte)
///   0: Vertical arrangement/horizontal mirroring (CIRAM A10 = PPU A11)
///      Horizontal arrangement/vertical mirroring (CIRAM A10 = PPU A10)
///   1: Cartridge contains battery-backed PRG RAM ($6000-7FFF)
///   2: 512-byte trainer at $7000-$71FF (stored before PRG data
///   3: Four-screen VRAM
/// 4-7: Lower part of mapper number
///
/// http://wiki.nesdev.com/w/index.php/INES#Flags_6
#[derive(Debug)]
pub struct Flags6 {
	pub horizontal_arrangement: bool,
	pub battery_backed_prg_ram: bool,
	pub trainer: bool,
	pub four_screen_vram: bool,
	pub mapper_lower: u8
}

impl Flags6 {
	fn new(data: &u8) -> Flags6 {
		Flags6 {
			horizontal_arrangement: data & 0b1 == 0b1,
			battery_backed_prg_ram: data & 0b10 == 0b10,
			trainer: data & 0b100 == 0b100,
			four_screen_vram: data & 0b1000 == 0b1000,
			mapper_lower: data >> 4
		}
	}
}

/// Flags 7 (1 Byte)
///   0: vs Unisystem
///   1: PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
/// 2-3: If equal to 2, flags 8-15 are in NES 2.0 format
/// 4-7: Upper part of mapper number
///
/// http://wiki.nesdev.com/w/index.php/INES#Flags_7
#[derive(Debug)]
pub struct Flags7 {
	pub vs_unisystem: bool,
	pub playchoice_10: bool,
	pub ines_2: bool,
	pub mapper_upper: u8
}

impl Flags7 {
	fn new(data: &u8) -> Flags7 {
		Flags7 {
			vs_unisystem: data & 0b1 == 0b1,
			playchoice_10: data & 0b10 == 0b10,
			ines_2: data >> 6 == 2u8,
			mapper_upper: data >> 4
		}
	}
}

/// Rom data and ines header
#[derive(Debug)]
pub struct Rom {
	pub header: Header,
	pub data: Vec<u8>
}

impl Rom {
	pub fn open<P: AsRef<Path>>(path: P) -> Result<Rom> {
		let mut file = try!(File::open(path));

		// Load header data
		let mut header_data: [u8; 16] = [0; 16];
		match file.read_exact(&mut header_data) {
			Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to read header!")),
			_ => ()
		}
		let header = try!(Header::new(&header_data));

		// Load all data after header
		let mut data = Vec::<u8>::new();
		try!(file.read_to_end(&mut data));

		Ok(Rom {
			header: header,
			data: data
		})
	}
}
