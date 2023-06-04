// mapper1 = DQ3, DQ4
// mapper2 = DQ2
// mapper3 = bad apple

use log::{info, trace};

use crate::rom::Rom;

pub fn mapper_from_rom(rom: &Rom) -> Box<dyn Mapper> {
    match rom.mapper {
        0 => Box::new(Mapper0::new()),
        1 => Box::new(Mapper1::new(rom.prg_rom.len())),
        _ => panic!("mapper = {} is not support.", rom.mapper),
    }
}

pub trait Mapper {
    fn write(&mut self, addr: u16, data: u8);
    fn mirror_prg_rom_addr(&self, addr: usize) -> usize;
    fn mirror_chr_rom_addr(&self, addr: usize) -> usize;
}

pub struct Mapper0 {}

impl Mapper0 {
    pub fn new() -> Self {
        Mapper0 {}
    }
}

impl Mapper for Mapper0 {
    fn write(&mut self, addr: u16, data: u8) {}
    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        if
        /* self.prg_rom.len() == 0x4000 && */
        addr >= 0xC000 {
            // mirror if needed
            addr - 0x4000
        } else {
            addr
        }
    }
    fn mirror_chr_rom_addr(&self, addr: usize) -> usize {
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
                    info!("MAPPER1: CHR_BANK0={:02X}", v)
                }
                0xC000..=0xDFFF => {
                    self.chr_bank1 = v;
                    info!("MAPPER1: CHR_BANK1={:02X}", v)
                }
                0xE000..=0xFFFF => {
                    self.prg_bank = v;
                    info!("MAPPER1: PRG_BANK={:02X}", v)
                }
                _ => panic!("can't be"),
            };
            self.reset_shift_register();
        }
    }

    fn mirror_prg_rom_addr(&self, addr: usize) -> usize {
        // FIXME ミラーリング

        // FIXME とりあえずMMC1Bとして処理。
        let num = self.prg_bank as usize & 0x0F;

        let bank_len = 16 * 1024;
        let last_bank = self.prg_rom_size / bank_len - 1;

        match self.prg_rom_bank_mode {
            PrgRomBankMode::IgnoreLowerBit => addr + ((num & 0x01) * bank_len),
            PrgRomBankMode::FirstBankIs8 => match addr {
                0x8000..=0xBFFF => {
                    return addr;
                }
                0xC000..=0xFFFF => {
                    return addr - bank_len + (num * bank_len);
                }
                _ => panic!("can't be"),
            },
            PrgRomBankMode::LastBankIsC => match addr {
                0x8000..=0xBFFF => {
                    return addr + (num * bank_len);
                }
                0xC000..=0xFFFF => {
                    return addr - bank_len + bank_len * last_bank; // FIXME LAST BANKは最大容量がわからないと設定できない。とりあえず固定。
                }
                _ => panic!("can't be"),
            },
        }
    }

    fn mirror_chr_rom_addr(&self, addr: usize) -> usize {
        let bank_len = 4 * 1024;

        match self.chr_rom_bank_mode {
            ChrRomBankMode::Switch8Kb => {
                // chr_bank0をみて、chr_bank1は無視。
                let num = self.chr_bank0 as usize & 0x0E;
                return addr + num * bank_len;
            }
            ChrRomBankMode::Switch4Kb => {
                match addr {
                    0x0000..=0x0FFF => {
                        // chr_bank0
                        let num = self.chr_bank0 as usize & 0x0F;
                        return addr + num * bank_len;
                    }
                    0x1000..=0x1FFF => {
                        // chr_bank1
                        let num = self.chr_bank1 as usize & 0x0F;
                        return addr - bank_len + num * bank_len;
                    }
                    _ => panic!("can't be"),
                }
            }
        }
    }
}
