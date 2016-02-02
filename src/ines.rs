
/// Identifier should always be the first 4 bytes of iNES header
const IDENTIFIER: &'static [ &'static u8 ] = &[0x4E, 0x45, 0x53, 0x1A]; // NES

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
struct Header {
	prg_rom_size: u8,
	chr_rom_size: u8,
	prg_ram_size: u8,
	flags_6: u8, // TODO: Use bitflags!
	flags_7: u8,
	flags_9: u8,
	flags_10: u8
}

/// iNES (.nes) file format
/// 	Note: Ignoring trainer
struct Ines {
	header: Header,
	prg_rom: Vec<u8>,
	chr_rom: Vec<u8>,
	inst_rom: Vec<u8>,
	p_rom: Vec<u8>
}
