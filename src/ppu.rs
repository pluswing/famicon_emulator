use bitflags::bitflags;
use log::{debug, info, trace};

use crate::{cpu::IN_TRACE, rom::Mirroring};

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

    // Mask 0x2001
    mask: MaskRegister,

    // Status 0x2002
    status: StatusRegister,

    // Scroll 0x2005
    pub scroll: ScrollRegister,

    cycles: usize,
    scanline: usize,
    pub nmi_interrupt: Option<i32>,
    pub clear_nmi_interrupt: bool,

    // 描画中にパレットテーブルを書き換えることが可能なので、その対応。
    // 書き込まれた時点でのscanlineとその時のパレットのスナップショットを持っておき、
    // レンダリング時に、その履歴を参照して描画することで実現。
    pub scanline_palette_indexes: Vec<usize>,
    pub scanline_palette_tables: Vec<[u8; 32]>,
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
            scroll: ScrollRegister::new(),
            internal_data_buf: 0,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: None,
            clear_nmi_interrupt: false,
            scanline_palette_indexes: vec![],
            scanline_palette_tables: vec![],
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        if !unsafe { IN_TRACE } {
            self.increment_vram_addr();
        }

        //        debug!("WRITE PPU: {:04X} {:02X}", addr, value);

        match addr {
            0..=0x1FFF => {
                // FIXME
                debug!("write CHR_ROM {:04X} => {:02X}", addr, value);
                // self.chr_rom[addr as usize] = value;
                // self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x2000..=0x2FFF => {
                trace!(
                    "WRITE PPU_VRAM {:04X} {:02X} => ({:02X})",
                    addr,
                    self.mirror_vram_addr(addr) as usize,
                    value
                );
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3EFF => {
                trace!(
                    "WRITE PPU_VRAM MIRROR {:04X} {:02X} => ({:02X})",
                    addr,
                    self.mirror_vram_addr(addr) as usize,
                    value
                );
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3F00..=0x3F1F => {
                debug!(
                    "WRITE PALATTE {:04X} {:02X} => ({:02X}) SL={}",
                    addr,
                    self.mirror_palette_addr(addr) as usize,
                    value,
                    self.scanline
                );
                self.write_palette_table(addr, value)
            }
            0x3F20..=0x3FFF => {
                debug!(
                    "WRITE PALATTE MIRROR {:04X} {:02X} => ({:02X}) SL={}",
                    addr,
                    self.mirror_palette_addr(addr) as usize,
                    value,
                    self.scanline
                );
                self.write_palette_table(addr, value)
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    fn write_palette_table(&mut self, addr: u16, value: u8) {
        let addr = self.mirror_palette_addr(addr) as usize;

        // palette_tableには最新情報を入れておく。
        self.palette_table[addr] = value;

        let scanline = self.scanline;
        let last_scanline = self.scanline_palette_indexes.last().unwrap_or(&0);
        if *last_scanline != scanline {
            self.scanline_palette_indexes.push(scanline);
            self.scanline_palette_tables
                .push(self.palette_table.clone());
        } else {
            self.scanline_palette_tables.pop();
            self.scanline_palette_tables
                .push(self.palette_table.clone());
        }
    }

    fn clear_palette_table_histories(&mut self) {
        self.scanline_palette_indexes = vec![];
        self.scanline_palette_tables = vec![];
    }

    pub fn read_palette_table(&self, scanline: usize) -> &[u8; 32] {
        if self.scanline_palette_indexes.is_empty() {
            return &self.palette_table;
        }

        let mut index = 0;
        for (i, s) in self.scanline_palette_indexes.iter().enumerate() {
            if *s > scanline {
                break;
            }
            index = i
        }

        info!("RPT SL={}, INDEX={}", scanline, index);

        let table = &self.scanline_palette_tables[index];

        info!("  => {:?}", table);

        table
    }

    fn mirror_palette_addr(&self, addr: u16) -> u16 {
        // see: https://taotao54321.hatenablog.com/entry/2017/04/11/115205
        let addr = addr & 0x1F;
        match addr {
            0x10 => 0x00,
            0x14 => 0x04,
            0x18 => 0x08,
            0x1C => 0x0C,
            _ => addr,
        }
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(value);
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn read_status(&mut self) -> u8 {
        // スクロール ($2005)  PPUSTATUSを読み取ってアドレス ラッチをリセットした後
        if unsafe { IN_TRACE } {
            self.status.bits()
        } else {
            self.scroll.reset();
            let bits = self.status.bits();
            self.status.reset_vblank_status();
            self.clear_nmi_interrupt = true;
            bits
        }
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
        debug!("OAM: {:04X} => {:02X}", self.oam_addr, value);
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1)
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn write_to_oam_dma(&mut self, values: [u8; 256]) {
        debug!("OAM DMA: ADDR:{:02X}", self.oam_addr);
        debug!("{:?}", values);
        self.oam_data = values;
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.set(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        if !unsafe { IN_TRACE } {
            self.increment_vram_addr();
        }
        debug!("READ PPU: {:04X}", addr);

        match addr {
            0..=0x1FFF => {
                if unsafe { IN_TRACE } {
                    self.internal_data_buf
                } else {
                    let result = self.internal_data_buf;
                    self.internal_data_buf = self.chr_rom[addr as usize];
                    result
                }
            }
            0x2000..=0x2FFF => {
                if unsafe { IN_TRACE } {
                    self.internal_data_buf
                } else {
                    let result = self.internal_data_buf;
                    self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                    result
                }
            }
            0x3000..=0x3EFF => {
                if unsafe { IN_TRACE } {
                    self.internal_data_buf
                } else {
                    let result = self.internal_data_buf;
                    self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                    result
                }
            }
            0x3F00..=0x3F1F => {
                if unsafe { IN_TRACE } {
                    self.internal_data_buf
                } else {
                    self.internal_data_buf =
                        self.palette_table[self.mirror_palette_addr(addr) as usize];
                    self.internal_data_buf
                }
            }
            0x3F20..=0x3FFF => {
                // TODO
                0
            }
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
            if self.is_sprite_zero_hit(self.cycles) {
                self.status.set_sprite_zero_hit(true);
            }

            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                self.status.set_vblank_status(true);
                self.status.set_sprite_zero_hit(false);
                if self.ctrl.generate_vblank_nmi() {
                    // self.status.set_vblank_status(true);
                    self.nmi_interrupt = Some(1);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank_status();
                self.nmi_interrupt = None;
                self.clear_palette_table_histories();
                return true;
            }

            if self.scanline == 257 {
                // OAMADDR は、プリレンダリングおよび表示可能なスキャンラインのティック 257 ～ 320 (スプライト タイルの読み込み間隔) のそれぞれの間に 0 に設定されます。
                self.oam_addr = 0;
            }
        }
        return false;
    }

    fn is_sprite_zero_hit(&self, cycle: usize) -> bool {
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

    pub fn background_pattern_addr(&self) -> u16 {
        if !self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn is_sprite_8x16_mode(&self) -> bool {
        self.contains(ControlRegister::SPRITE_SIZE)
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        // ignored in 8x16 mode

        if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x0000
        } else {
            0x1000
        }
    }

    pub fn nametable_addr(&self) -> u16 {
        match (
            self.contains(ControlRegister::NAMETABLE2),
            self.contains(ControlRegister::NAMETABLE1),
        ) {
            (false, false) => 0x2000,
            (false, true) => 0x2400,
            (true, false) => 0x2800,
            (true, true) => 0x2C00,
        }
    }
}

bitflags! {
  pub struct StatusRegister: u8 {
    const PPU_OPEN_BUS1       = 0b0000_0001;
    const PPU_OPEN_BUS2       = 0b0000_0010;
    const PPU_OPEN_BUS3       = 0b0000_0100;
    const PPU_OPEN_BUS4       = 0b0000_1000;
    const PPU_OPEN_BUS5       = 0b0001_0000; // VRAM状態
    const SPRITE_OVERFLOW     = 0b0010_0000;
    const SPRITE_ZERO_HIT     = 0b0100_0000;
    const VBLANK_HAS_STARTED  = 0b1000_0000;
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

    pub fn set_sprite_zero_hit(&mut self, value: bool) {
        self.set(StatusRegister::SPRITE_ZERO_HIT, value)
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

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SHOW_SPRITES)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

pub struct ScrollRegister {
    pub scroll_x: u8,
    pub scroll_y: u8,
    write_x: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            scroll_x: 0,
            scroll_y: 0,
            write_x: true,
        }
    }

    fn set(&mut self, data: u8) {
        if self.write_x {
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
        }
        self.write_x = !self.write_x;
    }

    pub fn reset(&mut self) {
        self.write_x = true;
    }
}
