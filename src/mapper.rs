use log::{info, warn};

use crate::rom::{Mirroring, Rom};

pub fn mapper_from_rom(rom: Rom) -> Box<dyn Mapper> {
    match rom.mapper {
        0 => Box::new(Mapper0::new(rom)), // SMB ...
        1 => Box::new(Mapper1::new(rom)), // DQ3 (1B), DQ4 (1B)
        2 => Box::new(Mapper2::new(rom)), // DQ2 (2H)
        4 => Box::new(Mapper4::new(rom)), // FF3 (4BH)
        _ => panic!("mapper = {} is not support.", rom.mapper),
    }
}

pub trait Mapper {
    fn write(&mut self, addr: u16, data: u8);
    fn prg_rom(&self, addr: usize) -> u8;
    fn chr_rom(&mut self, addr: usize) -> u8;
    fn mirroring(&self) -> Mirroring;
    fn is_chr_ram(&self) -> bool;
}

pub struct Mapper0 {
    rom: Rom,
}

impl Mapper0 {
    pub fn new(rom: Rom) -> Self {
        Mapper0 { rom }
    }
}

impl Mapper for Mapper0 {
    fn write(&mut self, _addr: u16, _data: u8) {}
    fn prg_rom(&self, addr: usize) -> u8 {
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0xC000 {
            // mirror if needed
            self.rom.prg_rom[addr - 0x4000]
        } else {
            self.rom.prg_rom[addr]
        }
    }
    fn chr_rom(&mut self, addr: usize) -> u8 {
        self.rom.chr_rom[addr]
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn is_chr_ram(&self) -> bool {
        self.rom.is_chr_ram
    }
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
    rom: Rom,

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
    pub fn new(rom: Rom) -> Self {
        Mapper1 {
            rom: rom,
            shift_register: 0,
            write_count: 0,
            mirroring: Mirroring::LOWER,
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
                        0 => Mirroring::LOWER,
                        1 => Mirroring::UPPER,
                        2 => Mirroring::VERTICAL,
                        3 => Mirroring::HORIZONTAL,
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

    fn prg_rom(&self, addr: usize) -> u8 {
        let num = self.prg_bank as usize & 0x0F;

        let bank_len = 16 * 1024;
        let last_bank = self.rom.prg_rom.len() / bank_len - 1;

        let mirrored_addr = match self.prg_rom_bank_mode {
            PrgRomBankMode::IgnoreLowerBit => (num & 0x0E) * bank_len,
            PrgRomBankMode::FirstBankIs8 => match addr {
                0x8000..=0xBFFF => addr,
                0xC000..=0xFFFF => num * bank_len,
                _ => panic!("can't be"),
            },
            PrgRomBankMode::LastBankIsC => match addr {
                0x8000..=0xBFFF => addr + num * bank_len,
                0xC000..=0xFFFF => addr - bank_len + last_bank * bank_len,
                _ => panic!("can't be"),
            },
        };
        self.rom.prg_rom[mirrored_addr - 0x8000]
    }

    fn chr_rom(&mut self, addr: usize) -> u8 {
        let bank_len = 4 * 1024;

        let mirrored_addr = match self.chr_rom_bank_mode {
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
        };
        self.rom.chr_rom[mirrored_addr]
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn is_chr_ram(&self) -> bool {
        self.rom.is_chr_ram
    }
}

pub struct Mapper2 {
    rom: Rom,

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
    pub fn new(rom: Rom) -> Self {
        Mapper2 {
            rom: rom,
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

    fn prg_rom(&self, addr: usize) -> u8 {
        let bank_len: usize = 8 * 1024;
        let num = self.prg_bank as usize & 0x0F;
        let last_bank = self.rom.prg_rom.len() / bank_len - 1;
        let mirrored_addr = match addr {
            0x8000..=0x9FFF => addr + bank_len * num,
            0xA000..=0xBFFF => (addr - bank_len * 1) + bank_len * (last_bank - 2),
            0xC000..=0xDFFF => (addr - bank_len * 2) + bank_len * (last_bank - 1),
            0xE000..=0xFFFF => (addr - bank_len * 3) + bank_len * (last_bank - 0),
            _ => panic!("can't be"),
        };
        self.rom.prg_rom[mirrored_addr - 0x8000]
    }

    fn chr_rom(&mut self, addr: usize) -> u8 {
        let bank_len: usize = 4 * 1024;
        let mirrored_addr = match addr {
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
        };

        self.rom.chr_rom[mirrored_addr]
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn is_chr_ram(&self) -> bool {
        self.rom.is_chr_ram
    }
}

pub struct Mapper4 {
    rom: Rom,

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
    pub fn new(rom: Rom) -> Self {
        Mapper4 {
            rom: rom,
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

    fn prg_rom(&self, addr: usize) -> u8 {
        let bank_len: usize = 16 * 1024;
        let num = self.prg_bank as usize & 0x0F;
        let last_bank = self.rom.prg_rom.len() / bank_len - 1;
        let mirrored_addr = match addr {
            0x8000..=0xBFFF => addr + bank_len * num,
            0xC000..=0xFFFF => addr - bank_len + bank_len * last_bank,
            _ => panic!("can't be"),
        };
        self.rom.prg_rom[mirrored_addr - 0x8000]
    }

    fn chr_rom(&mut self, addr: usize) -> u8 {
        let bank_len: usize = 4 * 1024;
        let mirrored_addr = match addr {
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
        };

        self.rom.chr_rom[mirrored_addr]
    }
    fn mirroring(&self) -> Mirroring {
        self.rom.screen_mirroring
    }
    fn is_chr_ram(&self) -> bool {
        self.rom.is_chr_ram
    }
}
