use bitflags::bitflags;

use crate::rom::Mirroring;

pub struct NesPPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],

    pub oam_addr: u8,
    pub oam_data: [u8; 256],

    pub mirroring: Mirroring,

    addr: AddrRegister,        // 0x2006 (0x2007)
    pub ctrl: ControlRegister, // 0x2000
    internal_data_buf: u8,

    // TODO Mask 0x2001
    mask: MaskRegister,

    // Status 0x2002
    status: StatusRegister,

    // TODO OAM Address 0x2003, 0x2004
    // TODO Scroll 0x2005
    // TODO OAM DMA 0x4014
    cycles: usize,
    scanline: usize,
    pub nmi_interrupt: Option<i32>,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom: chr_rom,
            mirroring: mirroring,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            oam_addr: 0,
            palette_table: [0; 32],
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
            status: StatusRegister::new(),
            mask: MaskRegister::new(),
            internal_data_buf: 0,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                // FIXME
            }
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => {
                // FIXME
            }
            0x3F00..=0x3FFF => {
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn read_status(&self) -> u8 {
        self.status.bits()
    }

    pub fn write_to_status(&mut self, value: u8) {
        self.status.update(value);
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1)
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_dma(&mut self, values: [u8; 256]) {
        self.oam_data = values;
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => panic!(
                "addr space 0x3000..0x3EFF is not expected to be used, requested = {} ",
                addr,
            ),
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10_1111_1111_1111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) => vram_index - 0x800,
            (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            if self.is_sprite_0_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }

            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                // self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    self.status.set_vblank_status(true);
                    // todo!("Should trigger NMI interupt")
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
                self.nmi_interrupt = None;
                return true;
            }

            if self.scanline == 257 {
                // OAMADDR は、プリレンダリングおよび表示可能なスキャンラインのティック 257 ～ 320 (スプライト タイルの読み込み間隔) のそれぞれの間に 0 に設定されます。
                self.oam_addr = 0;
            }
        }
        return false;
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as usize;
        let x = self.oam_data[3] as usize;
        (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
    }
}

pub struct AddrRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xFF) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3FFF {
            self.set(self.get() & 0b11_1111_1111_1111);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

bitflags! {
  pub struct ControlRegister: u8 {
    const NAMETABLE1               = 0b0000_0001;
    const NAMETABLE2               = 0b0000_0010;
    const VRAM_ADD_INCREMENT       = 0b0000_0100;
    const SPRITE_PATTERN_ADDR      = 0b0000_1000;
    const BACKGROUND_PATTERN_ADDR  = 0b0001_0000;
    const SPRITE_SIZE              = 0b0010_0000;
    const MASTER_SLAVE_SELECT      = 0b0100_0000;
    const GENERATE_NMI             = 0b1000_0000;
  }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }

    pub fn generate_vblank_nmi(&mut self) -> bool {
        let result = self.contains(ControlRegister::GENERATE_NMI);
        self.set(ControlRegister::GENERATE_NMI, true);
        return result;
    }

    pub fn bknd_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn is_sprt_8x16_mode(&self) -> bool {
        self.contains(ControlRegister::SPRITE_SIZE)
    }

    pub fn sprt_pattern_addr(&self) -> u16 {
        // ignored in 8x16 mode

        if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }
}

bitflags! {
  pub struct StatusRegister: u8 {
    const PPU_OPEN_BUS       = 0b0001_1111;
    const SPRITE_OVERFLOW    = 0b0010_0000;
    const SPRITE_ZERO_HIT    = 0b0100_0000;
    const VBLANK_HAS_STARTED = 0b1000_0000;
  }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn is_in_vblank(&mut self) -> bool {
        self.contains(StatusRegister::VBLANK_HAS_STARTED)
    }

    pub fn set_vblank_status(&mut self, value: bool) {
        self.set(StatusRegister::VBLANK_HAS_STARTED, value)
    }

    pub fn reset_vblank_status(&mut self) {
        self.set_vblank_status(false)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

bitflags! {
  pub struct MaskRegister: u8 {
    const GREYSCALE               = 0b0000_0001;
    const SHOW_BACKGROUND_IN_LEFT = 0b0000_0010;
    const SHOW_SPRITES_IN_LEFT    = 0b0000_0100;
    const SHOW_BACKGROUND         = 0b0000_1000;
    const SHOW_SPRITES            = 0b0001_0000;
    const EMPHASIZE_RED           = 0b0010_0000;
    const EMPHASIZE_GREEN         = 0b0100_0000;
    const EMPHASIZE_BLUE          = 0b1000_0000;
  }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}
