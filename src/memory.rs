
pub trait Memory {
	fn load(&self, address: u16) -> u8;
	fn store(&mut self, address: u16, value: u8);
}

pub struct ReadOnlyMemory {
	data: Vec<u8>
}

impl ReadOnlyMemory {
	pub fn new(capacity: usize) -> ReadOnlyMemory {
		ReadOnlyMemory {
			data: Vec::with_capacity(capacity)
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
