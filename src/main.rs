mod cpu;
mod mapper;
mod memory;
mod rom;

use cpu::CPU;
use rom::Rom;
use mapper::{ NRomPRG };
use std::env;

fn main() {
    // TODO: Impement real command line parsing, possibly with getopts or something similars
    let rom_file = env::args().nth(1).unwrap();

    let rom = Rom::open(rom_file).unwrap();
	println!("{:#?}", rom.header);

	let prg_rom = Box::new(NRomPRG::new(rom.header.clone(), rom.prg));

	let mut cpu = CPU::new(prg_rom);
	println!("Before power up: {}", cpu);
	cpu.power_up_with_pc_override(0xC000);
	println!("After power up: {}", cpu);
	cpu.run();
	println!("After run: {}", cpu);
}
