use crate::rom::{Mirroring, Rom};
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

pub struct Mapper0 {
    pub prg_rom: Vec<u8>,
}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 { prg_rom: vec![] }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        // 何もしない
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        let mut mirror_addr = addr - 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror if needed
            mirror_addr = mirror_addr % 0x4000;
        }
        self.prg_rom[mirror_addr as usize]
    }
}

pub struct Mapper1 {
    pub rom: Rom,
    prg_ram: [u8; 8192],

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
            prg_ram: [0xFF; 8192],
            shift_register: 0x10,
            shift_count: 0,

            control: 0x0C,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) {
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

    fn reset(&mut self) {
        self.shift_register = 0x10;
        self.shift_count = 0;
    }

    pub fn mirroring(&self) -> Mirroring {
        match self.control & 0x03 {
            2 => Mirroring::VERTICAL,
            3 => Mirroring::HORIZONTAL,
            _ => panic!("not support mirroring mode."),
        }
    }

    pub fn write_prg_ram(&mut self, addr: u16, data: u8) {
        self.prg_ram[addr as usize - 0x6000] = data;

        // FIXME　保存処理
        let mut file = File::create("save.dat").unwrap();
        file.write_all(&self.prg_ram).unwrap();
        file.flush().unwrap();
    }

    pub fn read_prg_ram(&self, addr: u16) -> u8 {
        self.prg_ram[addr as usize - 0x6000]
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
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

    pub fn write_chr_rom(&mut self, addr: u16, value: u8) {
        let mirrored_addr = self.chr_rom_addr(addr);
        self.rom.chr_rom[mirrored_addr] = value
    }

    pub fn read_chr_rom(&self, addr: u16) -> u8 {
        self.rom.chr_rom[self.chr_rom_addr(addr)]
    }

    fn chr_rom_addr(&self, addr: u16) -> usize {
        addr as usize
    }
}

pub struct Mapper2 {
    pub prg_rom: Vec<u8>,
    bank_select: u8,
}

impl Mapper2 {
    pub fn new() -> Self {
        Mapper2 {
            prg_rom: vec![],
            bank_select: 0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data;
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        let bank_len = 16 * 1024 as usize;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x8000..=0xBFFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            }
            0xC000..=0xFFFF => {
                // 最後のバンク固定
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            }
            _ => panic!("can't be"),
        }
    }
}
