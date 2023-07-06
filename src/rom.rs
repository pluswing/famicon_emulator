#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // NES^Z
const PRG_ROM_PAGE_SIZE: usize = 16 * 1024; // 16KiB
const CHR_ROM_PAGE_SIZE: usize = 8 * 1024; // 8KiB

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
    pub is_chr_ram: bool,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        // let ines_ver = (raw[7] >> 2) & 0b11;
        // if ines_ver != 0 {
        //     return Err("NES2.0 format is not supported".to_string());
        // }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let chr_rom = if chr_rom_size == 0 {
            // chr_rom_size=0の場合、8KBのCHR_RAMが存在する
            let blank_chr_ram: Vec<u8> = vec![0; CHR_ROM_PAGE_SIZE * 16];
            blank_chr_ram
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: chr_rom,
            mapper: mapper,
            screen_mirroring: screen_mirroring,
            is_chr_ram: chr_rom_size == 0,
        })
    }

    pub fn empty() -> Self {
        return Rom {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::VERTICAL,
            is_chr_ram: false,
        };
    }
}
