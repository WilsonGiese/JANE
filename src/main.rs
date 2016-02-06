mod cpu;
mod ines;

use cpu::CPU;
use ines::Ines;

fn main() {
	let mut cpu = CPU::default();
	cpu.power_up();
	println!("{}", cpu);

    let ines = Ines::open("roms/smb.nes").unwrap();
	println!("{:#?}", ines.header);

	println!("PRG ROM:");
	for i in 0..10 {
		println!("{:#X}", ines.prg_rom.get(i).unwrap());
	}

	println!("CHR ROM:");
	for i in 0..10 {
		println!("{:#X}", ines.chr_rom.get(i).unwrap());
	}
}
