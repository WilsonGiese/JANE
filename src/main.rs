mod cpu;

use cpu::CPU;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::Result;

fn main() {
	let mut cpu = CPU::default();
	cpu.power_up();
	println!("{:#?}", cpu);

    let buffer = read_rom("roms/smb.nes").unwrap();
	// Roms begin with first 3 bytes = "NES"
	println!("First 3 bytes:");
	for b in &buffer[0..3] {
		println!("{:#x} {}", b, *b as char);
	}
}

// Read bytes from NES rom from file
// Common ROM format is iNES and NES 2.0
// 		http://wiki.nesdev.com/w/index.php/INES
fn read_rom<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
	let mut buffer = Vec::<u8>::new();

	let mut rom = try!(File::open(path));
	try!(rom.read_to_end(&mut buffer));

	Ok(buffer)
}
