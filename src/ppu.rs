
pub struct PpuRegisters {
	pub ppuctrl: PpuCtrlRegister,
	pub ppumask: PpuMaskRegister,
	pub ppstatus: PpuStatusRegister,
	pub oamaddr: u8,
	pub oamdata: u8,
	pub ppuscroll: u8,
	pub ppuaddr: u8,
	pub ppudata: u8,
	pub oamdma: u8
}

pub struct PpuCtrlRegister {
	nmi_enabled: bool,
	master_slave_select: bool,
	sprite_size: (u8, u8),
	background_pattern_address: u16,
	vram_incrementer_select: bool,
	base_nametable_address: u16
}

pub struct PpuMaskRegister {
	greyscale: bool,
	show_background_leftmost_pixels: bool,
	show_sprites_leftmost_pixels: bool,
	show_background: bool,
	show_sprites: bool,
	emphasize_red: bool,
	emphasize_green: bool,
	emphasize_blue: bool
}

pub struct PpuStatusRegister {
	recent_least_significant_bits: u8,
	sprite_overflow: bool,
	sprite_hit: bool,
	vertical_blank_started: bool
}
