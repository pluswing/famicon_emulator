pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_talbe: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],

    pub mirroring: Mirroring,
}
