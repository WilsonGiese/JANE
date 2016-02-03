
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
struct Header {
	prg_rom_size: u8,
	chr_rom_size: u8,
	prg_ram_size: u8,
	flags6: Flags6,
	// TODO: Flags 7,9,10
}

impl Header {
	fn new(data: &[u8; 16]) -> Header {
		Header {
			prg_rom_size: data[4],
			chr_rom_size: data[5],
			prg_ram_size: data[8],
			flags6: Flags6::new(&data[6])
		}
	}
}

/// Flags 6
///   0: vs Unisystem
///   1: PlayChoice-10 (8KB of Hint Screen data stored after CHR data)
/// 2-3: If equal to 2, flags 8-15 are in NES 2.0 format
///   3: four-screen VRAM
/// 4-7: Ignored (part of mapper number)
///
/// http://wiki.nesdev.com/w/index.php/INES#Flags_6
#[derive(Debug)]
struct Flags6 {
	horizontal_arrangement: bool,
	battery_backed_prg_ram: bool,
	trainer: bool,
	four_screen_vram: bool
}

impl Flags6 {
	fn new(data: &u8) -> Flags6 {
		Flags6 {
			horizontal_arrangement: data & 0b1 == 1,
			battery_backed_prg_ram: data & 0b10 == 1,
			trainer: data & 0b100 == 1,
			four_screen_vram: data & 0b1000 == 1
		}
	}
}

/// iNES (.nes) file format
/// 	Note: Ignoring trainer
#[derive(Debug)]
pub struct Ines {
	header: Header,
	// prg_rom: Vec<u8>,
	// chr_rom: Vec<u8>,
	// inst_rom: Vec<u8>,
	// p_rom: Vec<u8>
}

impl Ines {
	pub fn open<P: AsRef<Path>>(path: P) -> Result<Ines> {
		let mut header_data: [u8; 16] = [0; 16];

		let mut file = try!(File::open(path));
		try!(file.read_exact(&mut header_data));

		// First 4 bytes must be the identifier for iNes files
		if header_data[0..4] != IDENTIFIER {
			return Err(Error::new(ErrorKind::Other, format!("File is not in iNES file format!")));
		}

		Ok(Ines {
			header: Header::new(&header_data)
		})
	}
}
