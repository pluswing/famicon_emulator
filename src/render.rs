use crate::frame::Frame;
use crate::palette;
use crate::ppu::NesPPU;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.bknd_pattern_addr();
    for i in 0..0x03C0 {
        let tile = ppu.vram[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
        let palette = bg_pallette(ppu, tile_x, tile_y);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[0x01],
                    1 => palette::SYSTEM_PALLETE[0x23],
                    2 => palette::SYSTEM_PALLETE[0x27],
                    3 => palette::SYSTEM_PALLETE[0x30],
                    // 0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                    // 1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    // 2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    // 3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
            }
        }
    }
}

fn bg_pallette(ppu: &NesPPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x03C0 + attr_table_idx];

    let pallet_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        _ => panic!("should not happen"),
    };

    let pallette_start: usize = 1 + (pallet_idx as usize) * 4;
    [
        ppu.palette_table[0],
        ppu.palette_table[pallette_start],
        ppu.palette_table[pallette_start + 1],
        ppu.palette_table[pallette_start + 2],
    ]
}
