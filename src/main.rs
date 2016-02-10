mod cpu;
mod rom;
mod memory;

use cpu::{ CPU, NRom };
use rom::Rom;

fn main() {
    let rom = Box::new(Rom::open("roms/smb.nes").unwrap());
	println!("{:#?}", rom.header);

	let mapped_memory = Box::new(NRom::new(rom).unwrap());

	let mut cpu = CPU::new(mapped_memory);
	println!("Before power up: {}", cpu);
	cpu.power_up();
	println!("After power up: {}", cpu);
	cpu.run();
	println!("After run: {}", cpu);
}
