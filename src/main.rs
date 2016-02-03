mod cpu;
mod ines;

use cpu::CPU;
use ines::Ines;

fn main() {
	let mut cpu = CPU::default();
	cpu.power_up();
	println!("{}", cpu);

    let ines = Ines::open("roms/smb.nes").unwrap();
	println!("{:#?}", ines);
}
