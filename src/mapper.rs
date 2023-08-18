use crate::rom::{Mirroring, Rom};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

pub fn create_mapper(rom: Rom) -> Box<dyn Mapper> {
    let mut mapper: Box<dyn Mapper> = match rom.mapper {
        0 => Box::new(Mapper0::new()),
        1 => Box::new(Mapper1::new()),
        2 => Box::new(Mapper2::new()),
        3 => Box::new(Mapper3::new()),
        _ => panic!("not support mapper."),
    };
    mapper.set_rom(rom);
    return mapper;
}

pub trait Mapper: Send {
    fn set_rom(&mut self, rom: Rom);
    fn is_chr_ram(&mut self) -> bool;
    fn write(&mut self, addr: u16, data: u8);
    fn mirroring(&self) -> Mirroring;
    fn write_prg_ram(&mut self, addr: u16, data: u8);
    fn read_prg_ram(&self, addr: u16) -> u8;
    fn load_prg_ram(&mut self, raw: &Vec<u8>);
    fn read_prg_rom(&self, addr: u16) -> u8;
    fn write_chr_rom(&mut self, addr: u16, value: u8);
    fn read_chr_rom(&self, addr: u16) -> u8;
}

pub struct Mapper0 {
    pub rom: Rom,
}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 { rom: Rom::empty() }
    }
}

impl Mapper for Mapper0 {
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom
    }
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn write(&mut self, addr: u16, data: u8) {}
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn write_prg_ram(&mut self, addr: u16, data: u8) {}
    fn read_prg_ram(&self, addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, raw: &Vec<u8>) {}
    fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut mirror_addr = addr - 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror if needed
            mirror_addr = mirror_addr % 0x4000;
        }
        self.rom.prg_rom[mirror_addr as usize]
    }
    fn write_chr_rom(&mut self, addr: u16, value: u8) {}
    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
}

pub struct Mapper1 {
    pub rom: Rom,
    prg_ram: Vec<u8>,

    shift_register: u8,
    shift_count: u8,

    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
}

// SUROM
impl Mapper1 {
    pub fn new() -> Self {
        Mapper1 {
            rom: Rom::empty(),
            prg_ram: vec![0xFF; 8192],
            shift_register: 0x10,
            shift_count: 0,

            control: 0x0C,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }

    fn reset(&mut self) {
        self.shift_register = 0x10;
        self.shift_count = 0;
    }
}

impl Mapper for Mapper1 {
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom
    }
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn write(&mut self, addr: u16, data: u8) {
        if data & 0x80 != 0 {
            self.reset();
            return;
        }

        self.shift_register = self.shift_register >> 1;
        self.shift_register = self.shift_register | ((data & 0x01) << 4);
        self.shift_count += 1;

        if self.shift_count == 5 {
            match addr {
                0x8000..=0x9FFF => self.control = self.shift_register,
                0xA000..=0xBFFF => self.chr_bank0 = self.shift_register,
                0xC000..=0xDFFF => self.chr_bank1 = self.shift_register,
                0xE000..=0xFFFF => self.prg_bank = self.shift_register,
                _ => panic!("can't be"),
            }
            self.reset();
        }
    }

    fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
            _ => panic!("not support mirroring mode."),
        }
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        self.prg_ram[addr as usize - 0x6000] = data;

        // FIXME　保存処理
        let mut file = File::create("save.dat").unwrap();
        file.write_all(&self.prg_ram).unwrap();
        file.flush().unwrap();
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize - 0x6000]
    }

    fn load_prg_ram(&mut self, raw: &Vec<u8>) {
        self.prg_ram = raw.to_vec()
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_len = 16 * 1024 as usize;
        let bank_max = self.rom.prg_rom.len() / bank_len;
        let mut bank = self.prg_bank & 0x0F;
        let mut first_bank = 0x00;
        let mut last_bank = bank_max - 1;

        if self.chr_bank0 & 0x10 != 0 {
            bank = bank | 0x10;
            first_bank = first_bank | 0x10;
            last_bank = last_bank | 0x10;
        } else {
            bank = bank & 0x0F;
            first_bank = first_bank & 0x0F;
            last_bank = last_bank & 0x0F;
        }

        match (self.control & 0x0C) >> 2 {
            0 | 1 => {
                // バンク番号の下位ビットを無視して、32 KB を $8000 に切り替えます。
                self.rom.prg_rom
                    [(addr as usize - 0x8000 + bank_len * (bank & 0x1E) as usize) as usize]
            }
            2 => {
                // 最初のバンクを $8000 に固定し、16 KB バンクを $C000 に切り替えます。
                match addr {
                    0x8000..=0xBFFF => {
                        self.rom.prg_rom[addr as usize - 0x8000 + bank_len * first_bank]
                    }
                    0xC000..=0xFFFF => {
                        self.rom.prg_rom
                            [(addr as usize - 0xC000 + bank_len * bank as usize) as usize]
                    }
                    _ => panic!("can't be"),
                }
            }
            3 => {
                // 最後のバンクを $C000 に固定し、16 KB バンクを $8000 に切り替えます)
                match addr {
                    0x8000..=0xBFFF => {
                        self.rom.prg_rom
                            [(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
                    }
                    0xC000..=0xFFFF => {
                        self.rom.prg_rom[addr as usize - 0xC000 + bank_len * last_bank]
                    }
                    _ => panic!("can't be"),
                }
            }
            _ => panic!("can't be"),
        }
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value
    }

    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
}

pub struct Mapper2 {
    pub rom: Rom,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 {
            rom: Rom::empty(),
            bank_select: 0,
        }
    }
}

impl Mapper for Mapper2 {
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom
    }
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data;
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {}
    fn read_prg_ram(&self, addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_len = 16 * 1024 as usize;
        let bank_max = self.rom.prg_rom.len() / bank_len;
        match addr {
            0x8000..=0xBFFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.rom.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            }
            0xC000..=0xFFFF => {
                // 最後のバンク固定
                self.rom.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            }
            _ => panic!("can't be"),
        }
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[addr as usize]
    }
}

pub struct Mapper3 {
    pub rom: Rom,
    bank_select: u8,
}

impl Mapper3 {
    pub fn new() -> Self {
        Mapper3 {
            rom: Rom::empty(),
            bank_select: 0,
        }
    }
}

impl Mapper for Mapper3 {
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom
    }
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data;
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {}
    fn read_prg_ram(&self, addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.rom.prg_rom[(addr as usize - 0x8000)]
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        let bank_len = 8 * 1024 as usize;
        let bank = self.bank_select & 0x03;
        self.rom.chr_rom[(addr as usize + bank_len * bank as usize) as usize]
    }
}

pub struct Mapper4 {
    pub rom: Rom,
    // pub bank_select: u8,
    // ..
}

impl Mapper4 {
    pub fn new() -> Self {
        Mapper4 { rom: Rom::empty() }
    }
}

impl Mapper for Mapper4 {
    fn set_rom(&mut self, rom: Rom) {
        self.rom = rom
    }
    fn is_chr_ram(&mut self) -> bool {
        self.rom.is_chr_ram
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x8000..=0x9FFF => {
                if addr & 0x01 == 0 {
                    // バンクセレクト ($8000-$9FFE、偶数)
                } else {
                    // 銀行データ ($8001 ～ $9FFF、奇数)
                }
            }
            0xA000..=0xBFFE => {
                if addr & 0x01 == 0 {
                    // ミラーリング ($A000-$BFFE、偶数)
                } else {
                    // PRG RAM 保護 ($A001-$BFFF、奇数)
                }
            }
            0xC000..=0xDFFE => {
                if addr & 0x01 == 0 {
                    // IRQ ラッチ ($C000-$DFFE、偶数)
                } else {
                    // IRQ リロード ($C001-$DFFF、奇数)
                }
            }
            0xE000..=0xFFFE => {
                if addr & 0x01 == 0 {
                    // IRQ 無効化 ($E000-$FFFE、偶数)
                } else {
                    // IRQ イネーブル ($E001-$FFFF、奇数)
                }
            }
        }
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {}
    fn read_prg_ram(&self, addr: u16) -> u8 {
        0
    }
    fn load_prg_ram(&mut self, raw: &Vec<u8>) {}

    fn read_prg_rom(&self, addr: u16) -> u8 {
        // TODO
        /*
        $8000-$9FFF	R6	(-2)
        $A000-$BFFF	R7	R7
        $C000-$DFFF	(-2)	R6
        $E000-$FFFF	(-1)	(-1)
         */
    }

    fn write_chr_rom(&mut self, addr: u16, value: u8) {
        self.rom.chr_rom[addr as usize] = value;
    }
    fn read_chr_rom(&self, addr: u16) -> u8 {
        // TODO
        /*
        $0000-$03FF	R0	R2
        $0400-$07FF	R3
        $0800-$0BFF	R1	R4
        $0C00-$0FFF	R5
        $1000-$13FF	R2	R0
        $1400-$17FF	R3
        $1800-$1BFF	R4	R1
        $1C00-$1FFF	R5
         */
    }
}
