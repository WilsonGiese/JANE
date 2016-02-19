
pub trait Memory {
	fn load(&self, address: u16) -> u8;
	fn store(&mut self, address: u16, value: u8);

	fn loadw(&self, address: u16) -> u16 {
		self.load(address) as u16 | (self.load(address + 1) as u16) << 8
	}

	fn storew(&mut self, address: u16, value: u16) {
		self.store(address, value as u8);
		self.store(address + 1, (value >> 8) as u8); 
	}
}

pub struct ReadOnlyMemory {
	data: Box<Vec<u8>>
}

impl ReadOnlyMemory {
	pub fn new(data: Box<Vec<u8>>) -> ReadOnlyMemory {
		ReadOnlyMemory {
			data: data
		}
	}
}

impl Memory for ReadOnlyMemory {
	fn load(&self, address: u16) -> u8 {
		self.data[address as usize]
	}

	fn store(&mut self, address: u16, value: u8) {
		panic!("Can't write to read only memory!")
	}
}

pub struct ReadWriteMemory {
	data: Vec<u8>
}

impl ReadWriteMemory {
	pub fn new(capacity: usize) -> ReadWriteMemory {
		ReadWriteMemory {
			data: Vec::with_capacity(capacity)
		}
	}
}

impl Memory for ReadWriteMemory {
	fn load(&self, address: u16) -> u8 {
		self.data[address as usize]
	}

	fn store(&mut self, address: u16, value: u8) {
		self.data[address as usize] = value;
	}
}
