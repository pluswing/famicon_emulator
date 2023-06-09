// mapper1 = DQ3, DQ4
// mapper2 = DQ2
// mapper3 = bad apple

use log::{info, trace, warn};

use crate::rom::Rom;

pub fn mapper_from_rom(rom: &Rom) -> Box<dyn Mapper> {
    match rom.mapper {
        0 => Box::new(Mapper0::new(rom.prg_rom.len())), // SMB ...
        1 => Box::new(Mapper1::new(rom.prg_rom.len())), // DQ3 (1B), DQ4 (1B)
        2 => Box::new(Mapper2::new(rom.prg_rom.len())), // DQ2 (2H)
        4 => Box::new(Mapper4::new(rom.prg_rom.len())), // FF3 (4BH)
        _ => panic!("mapper = {} is not support.", rom.mapper),
    }
}

pub trait Mapper {
    fn write(&mut self, addr: u16, data: u8);
    fn mirror_prg_rom_addr(&self, addr: usize) -> usize;
    fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize;
}

pub struct Mapper0 {
    prg_rom_size: usize,
}

impl Mapper0 {
    pub fn new(prg_rom_size: usize) -> Self {
        Mapper0 {
            prg_rom_size: prg_rom_size,
        }
    }
}

impl Mapper for Mapper0 {
    fn write(&mut self, addr: u16, data: u8) {}
    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        if self.prg_rom_size == 0x4000 && addr >= 0xC000 {
            // mirror if needed
            addr - 0x4000
        } else {
            addr
        }
    }
    fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize {
        addr
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum Mirroring {
    Lower,      // 0: 1 画面、下位バンク
    Upper,      // 1: 1 画面、上位バンク
    Vertical,   // 2: 垂直
    Horizontal, // 3: 水平
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum PrgRomBankMode {
    IgnoreLowerBit, // 0、 1: バンク番号の下位ビットを無視して、32 KB を $8000 に切り替えます。
    FirstBankIs8,   // 2: 最初のバンクを $8000 に固定し、16 KB バンクを $C000 に切り替えます。
    LastBankIsC,    // 3: 最後のバンクを $C000 に固定し、16 KB バンクを $8000 に切り替えます
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum ChrRomBankMode {
    Switch8Kb, // 0: 一度に 8 KB を切り替え
    Switch4Kb, // 1: 2 つの別々の 4 KB バンクを切り替え
}

pub struct Mapper1 {
    prg_rom_size: usize,

    shift_register: u8,
    write_count: u8,

    // $8000-$9FFF
    // control: u8,
    mirroring: Mirroring,
    prg_rom_bank_mode: PrgRomBankMode,
    chr_rom_bank_mode: ChrRomBankMode,

    // $A000-$BFFF
    chr_bank0: u8,
    // $C000-$DFFF
    chr_bank1: u8,
    // $E000-$FFFF
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new(prg_rom_size: usize) -> Self {
        Mapper1 {
            prg_rom_size: prg_rom_size,
            shift_register: 0,
            write_count: 0,
            mirroring: Mirroring::Lower,
            prg_rom_bank_mode: PrgRomBankMode::LastBankIsC,
            chr_rom_bank_mode: ChrRomBankMode::Switch8Kb,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
        }
    }

    fn reset_shift_register(&mut self) {
        self.shift_register = 0;
        self.write_count = 0;
    }
}

impl Mapper for Mapper1 {
    fn write(&mut self, addr: u16, data: u8) {
        if data & 0x80 != 0 {
            self.reset_shift_register();
            return;
        }

        self.shift_register = self.shift_register >> 1;
        self.shift_register = self.shift_register | ((data & 0x01) << 4);
        self.write_count += 1;

        if self.write_count == 5 {
            let v = self.shift_register;
            match addr {
                0x8000..=0x9FFF => {
                    self.chr_rom_bank_mode = match (v & 0x10) >> 4 {
                        0 => ChrRomBankMode::Switch8Kb,
                        1 => ChrRomBankMode::Switch4Kb,
                        _ => panic!("can't be"),
                    };
                    self.prg_rom_bank_mode = match (v & 0x0C) >> 2 {
                        0 | 1 => PrgRomBankMode::IgnoreLowerBit,
                        2 => PrgRomBankMode::FirstBankIs8,
                        3 => PrgRomBankMode::LastBankIsC,
                        _ => panic!("can't be"),
                    };
                    self.mirroring = match v & 0x03 {
                        0 => Mirroring::Lower,
                        1 => Mirroring::Upper,
                        2 => Mirroring::Vertical,
                        3 => Mirroring::Horizontal,
                        _ => panic!("can't be"),
                    };
                    info!("MAPPER1: control: mirroring={:?}, prg_rom_bank_mode={:?}, chr_rom_bank_mode={:?}", self.mirroring, self.prg_rom_bank_mode, self.chr_rom_bank_mode);
                }
                0xA000..=0xBFFF => {
                    self.chr_bank0 = v;
                    info!("MAPPER1: CHR_BANK0={:02X}", v & 0x1F)
                }
                0xC000..=0xDFFF => {
                    self.chr_bank1 = v;
                    info!("MAPPER1: CHR_BANK1={:02X}", v & 0x1F)
                }
                0xE000..=0xFFFF => {
                    self.prg_bank = v;
                    info!("MAPPER1: PRG_BANK={:02X}", v & 0x0F)
                }
                _ => panic!("can't be"),
            };
            self.reset_shift_register();
        }
    }

    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        let num = self.prg_bank as usize & 0x0F;

        let bank_len = 16 * 1024;
        let last_bank = self.prg_rom_size / bank_len - 1;

        match self.prg_rom_bank_mode {
            PrgRomBankMode::IgnoreLowerBit => (num & 0x0E) * bank_len,
            PrgRomBankMode::FirstBankIs8 => match addr {
                0x8000..=0xBFFF => 0,
                0xC000..=0xFFFF => num * bank_len,
                _ => panic!("can't be"),
            },
            PrgRomBankMode::LastBankIsC => match addr {
                0x8000..=0xBFFF => addr + num * bank_len,
                0xC000..=0xFFFF => addr - bank_len + last_bank * bank_len,
                _ => panic!("can't be"),
            },
        }
    }

    fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize {
        let bank_len = 4 * 1024;

        match self.chr_rom_bank_mode {
            ChrRomBankMode::Switch8Kb => {
                // chr_bank0をみて、chr_bank1は無視。
                let num = self.chr_bank0 as usize & 0x1E;
                num * bank_len
            }
            ChrRomBankMode::Switch4Kb => match addr {
                0x0000..=0x0FFF => {
                    let num = self.chr_bank0 as usize & 0x1F;
                    num * bank_len
                }
                0x1000..=0x1FFF => {
                    let num = self.chr_bank1 as usize & 0x1F;
                    num * bank_len
                }
                _ => panic!("can't be"),
            },
        }
    }
}

pub struct Mapper2 {
    prg_rom_size: usize,

    prg_bank: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    latch0: u8,
    chr_bank2: u8,
    chr_bank3: u8,
    latch1: u8,
    mirroring: u8,
}

impl Mapper2 {
    pub fn new(prg_rom_size: usize) -> Self {
        Mapper2 {
            prg_rom_size: prg_rom_size,
            prg_bank: 0,
            chr_bank0: 0,
            chr_bank1: 0,
            latch0: 0xFD,
            chr_bank2: 0,
            chr_bank3: 0,
            latch1: 0xFD,
            mirroring: 0,
        }
    }
}

impl Mapper for Mapper2 {
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0xA000..=0xAFFF => {
                info!("MAPPER2 PRG_BANK={}", data & 0x0F);
                self.prg_bank = data;
            }
            0xB000..=0xBFFF => {
                info!("MAPPER2 CHR_BANK0={}", data & 0x1F);
                self.chr_bank0 = data;
            }
            0xC000..=0xCFFF => {
                info!("MAPPER2 CHR_BANK1={}", data & 0x1F);
                self.chr_bank1 = data;
            }
            0xD000..=0xDFFF => {
                info!("MAPPER2 CHR_BANK2={}", data & 0x1F);
                self.chr_bank2 = data;
            }
            0xE000..=0xEFFF => {
                info!("MAPPER2 CHR_BANK3={}", data & 0x1F);
                self.chr_bank3 = data;
            }
            0xF000..=0xFFFF => {
                self.mirroring = data;
            }
            _ => panic!("can't be"),
        }
    }

    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        let bank_len: usize = 8 * 1024;
        let num = self.prg_bank as usize & 0x0F;
        let last_bank = self.prg_rom_size / bank_len - 1;
        match addr {
            0x8000..=0x9FFF => addr + bank_len * num,
            0xA000..=0xBFFF => (addr - bank_len * 1) + bank_len * (last_bank - 2),
            0xC000..=0xDFFF => (addr - bank_len * 2) + bank_len * (last_bank - 1),
            0xE000..=0xFFFF => (addr - bank_len * 3) + bank_len * (last_bank - 0),
            _ => panic!("can't be"),
        }
    }

    fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize {
        let bank_len: usize = 4 * 1024;
        let ret = match addr {
            0x0000..=0x0FFF => match self.latch0 {
                0xFD => addr + bank_len * (self.chr_bank0 & 0x1F) as usize,
                0xFE => addr + bank_len * (self.chr_bank1 & 0x1F) as usize,
                _ => panic!("can't be"),
            },
            0x1000..=0x1FFF => match self.latch1 {
                0xFD => addr - bank_len + bank_len * (self.chr_bank2 & 0x1F) as usize,
                0xFE => addr - bank_len + bank_len * (self.chr_bank3 & 0x1F) as usize,
                _ => panic!("can't be"),
            },
            _ => panic!("can't be"),
        };

        match addr {
            0x0FD8 => self.latch0 = 0xFD,
            0x0FE8 => self.latch0 = 0xFE,
            0x1FD8 => self.latch1 = 0xFD,
            0x1FE8 => self.latch1 = 0xFE,
            _ => {}
        }

        ret
    }
}

pub struct Mapper4 {
    prg_rom_size: usize,

    prg_bank: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    latch0: u8,
    chr_bank2: u8,
    chr_bank3: u8,
    latch1: u8,
    mirroring: u8,
}

impl Mapper4 {
    pub fn new(prg_rom_size: usize) -> Self {
        Mapper4 {
            prg_rom_size: prg_rom_size,
            prg_bank: 0,
            chr_bank0: 0,
            chr_bank1: 0,
            latch0: 0xFD,
            chr_bank2: 0,
            chr_bank3: 0,
            latch1: 0xFD,
            mirroring: 0,
        }
    }
}

impl Mapper for Mapper4 {
    fn write(&mut self, addr: u16, data: u8) {
        info!("MAPPER4 w: {:04X}", addr);
        match addr {
            0x8000 => {
                self.prg_bank = data;
            }
            0xA000..=0xAFFF => {
                self.prg_bank = data;
            }
            0xB000..=0xBFFF => {
                self.chr_bank0 = data;
            }
            0xC000..=0xCFFF => {
                self.chr_bank1 = data;
            }
            0xD000..=0xDFFF => {
                self.chr_bank2 = data;
            }
            0xE000..=0xEFFF => {
                self.chr_bank3 = data;
            }
            0xF000..=0xFFFF => {
                self.mirroring = data;
            }
            _ => {
                warn!("MAPPER4 write access: {:04X} => {:04X}", addr, data)
                // panic!("can't be"),
            }
        }
    }

    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        let bank_len: usize = 16 * 1024;
        let num = self.prg_bank as usize & 0x0F;
        let last_bank = self.prg_rom_size / bank_len - 1;
        match addr {
            0x8000..=0xBFFF => addr + bank_len * num,
            0xC000..=0xFFFF => addr - bank_len + bank_len * last_bank,
            _ => panic!("can't be"),
        }
    }

    fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize {
        let bank_len: usize = 4 * 1024;
        let ret = match addr {
            0x0000..=0x0FFF => match self.latch0 {
                0xFD => addr + bank_len * (self.chr_bank0 & 0x1F) as usize,
                0xFE => addr + bank_len * (self.chr_bank1 & 0x1F) as usize,
                _ => panic!("can't be"),
            },
            0x1000..=0x1FFF => match self.latch1 {
                0xFD => addr - bank_len + bank_len * (self.chr_bank2 & 0x1F) as usize,
                0xFE => addr - bank_len + bank_len * (self.chr_bank3 & 0x1F) as usize,
                _ => panic!("can't be"),
            },
            _ => panic!("can't be"),
        };

        match addr {
            0x0FD8 => self.latch0 = 0xFD,
            0x0FE8 => self.latch0 = 0xFE,
            0x1FD8 => self.latch1 = 0xFD,
            0x1FE8 => self.latch1 = 0xFE,
            _ => {}
        }

        ret
    }
}
