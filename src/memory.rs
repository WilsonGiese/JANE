// Internal RAM (and mirrors)
const RAM_SIZE: usize = 0x2000;
const RAM_END: usize = 0x1FFF;

// PPU registers
const PPU_MEM_SIZE: usize = 0x2000;
const PPU_MEM_END: usize = 0x3FFF;

// APU and io registers
const APU_MEM_SIZE: usize = 0x20;
const APU_MEM_END: usize = 0x401F;

// Cartridge space (prg ram, etc)
const CARTRIDGE_MEM_SIZE: usize = 0xBFE0;
const CARTRIDGE_MEM_START: usize = 0x4020;

pub trait MemoryOps {
	fn load(&self, address: u16) -> u8;
	fn store(&mut self, address: u16, value: u8);
}

struct Memory {
	ram: Ram,
	cartridgeMem: CartridgeMem
}

impl MemoryOps for Memory {
	fn load(&self, address: u16) -> u8 {

		if address as usize <= RAM_END {
			self.ram.load(address)
		} else if address as usize <= PPU_MEM_END {
			panic!("PPU Load is not implemented!") // TODO
		} else if address as usize <=  APU_MEM_END {
			panic!("APU Load is not implemented!") // TODO
		} else {
			self.cartridgeMem.load(address)
		}
	}

	fn store(&mut self, address: u16, value: u8) {
		panic!("Unsupported operation: Memory.store")
	}
}

struct Ram {
	data: [u8; RAM_SIZE as usize]
}

impl MemoryOps for Ram {
	fn load (&self, address: u16) -> u8 {
		self.data[address as usize]
	}

	fn store(&mut self, address: u16, value: u8) {
		panic!("Unsupported operation: Ram.store")
	}
}

struct CartridgeMem {
	data: [u8; CARTRIDGE_MEM_SIZE as usize]
}

impl MemoryOps for CartridgeMem {
	fn load (&self, address: u16) -> u8 {
		self.data[address as usize - CARTRIDGE_MEM_START]
	}

	fn store(&mut self, address: u16, value: u8) {
		panic!("Unsupported operation: Ram.store")
	}
}
