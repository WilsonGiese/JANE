mod cpu;
mod rom;
mod memory;

use cpu::CPU;
use rom::Rom;

fn main() {
    let rom = Rom::open("roms/smb.nes").unwrap();
	println!("{:#?}", rom.header);

	let mut cpu = CPU::new();
	println!("Before power up: {}", cpu);
	cpu.power_up();
	println!("After power up: {}", cpu);
	cpu.run(rom);
	println!("After run: {}", cpu);
}
