use crate::frame::Frame;
use crate::palette;
use crate::ppu::NesPPU;
use crate::rom::Mirroring;

struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rect {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Rect {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }
}

pub fn render(ppu: &mut NesPPU, frame: &mut Frame) {
    // draw background
    let scroll_x = (ppu.scroll.scroll_x) as usize;
    let scroll_y = (ppu.scroll.scroll_y) as usize;

    let (main_name_table, second_name_table) =
        match (ppu.mapper.mirroring(), ppu.ctrl.nametable_addr()) {
            (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) => {
                (&ppu.vram[0x000..0x400], &ppu.vram[0x400..0x800])
            }
            (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) => {
                (&ppu.vram[0x400..0x800], &ppu.vram[0x000..0x400])
            }
            (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => {
                (&ppu.vram[0x000..0x400], &ppu.vram[0x400..0x800])
            }
            (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => {
                (&ppu.vram[0x400..0x800], &ppu.vram[0x000..0x400])
            }
            (_, _) => {
                panic!("Not supported mirroring type {:?}", ppu.mapper.mirroring());
            }
        };

    let screen_w = 256;
    let screen_h = 240;

    // 左上
    render_name_table(
        ppu,
        frame,
        main_name_table,
        Rect::new(scroll_x, scroll_y, screen_w, screen_h),
        -(scroll_x as isize),
        -(scroll_y as isize),
    );

    // 右下
    render_name_table(
        ppu,
        frame,
        second_name_table,
        Rect::new(0, 0, scroll_x, scroll_y),
        (screen_w - scroll_x) as isize,
        (screen_h - scroll_y) as isize,
    );

    // 左下
    render_name_table(
        ppu,
        frame,
        main_name_table,
        Rect::new(scroll_x, 0, screen_w, scroll_y),
        -(scroll_x as isize),
        (screen_h - scroll_y) as isize,
    );

    // 右上
    render_name_table(
        ppu,
        frame,
        second_name_table,
        Rect::new(0, scroll_y, scroll_x, screen_h),
        (screen_w - scroll_x) as isize,
        -(scroll_y as isize),
    );

    // draw sprites
    // TODO 8x16 mode
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_y = ppu.oam_data[i] as usize;
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let attr = ppu.oam_data[i + 2];
        let tile_x = ppu.oam_data[i + 3] as usize;

        let flip_vertical = (attr >> 7 & 1) == 1;
        let flip_horizontal = (attr >> 6 & 1) == 1;
        let palette_idx = attr & 0b11;
        let sprite_palette = sprite_palette(ppu, tile_y, palette_idx);

        let bank: u16 = ppu.ctrl.sprite_pattern_addr();

        let mut tile: [u8; 15] = [0; 15];
        let mut i = 0;
        for addr in (bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize {
            tile[i] = ppu.mapper.chr_rom(addr);
            i += 1
        }

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
            'ololo: for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => continue 'ololo, // skip coloring the pixel
                    1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
                    _ => panic!("can't be"),
                };

                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                }
            }
        }
    }
}

fn bg_pallette(
    ppu: &mut NesPPU,
    attribute_table: &[u8],
    tile_column: usize,
    tile_row: usize,
) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = attribute_table[attr_table_idx];

    let pallet_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        _ => panic!("should not happen"),
    };

    let pallette_start: usize = 1 + (pallet_idx as usize) * 4;
    let p = ppu.read_palette_table(tile_row * 8);
    [
        p[0],
        p[pallette_start],
        p[pallette_start + 1],
        p[pallette_start + 2],
    ]
}

fn sprite_palette(ppu: &mut NesPPU, tile_y: usize, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + (palette_idx * 4) as usize;
    let p = ppu.read_palette_table(tile_y);
    [0, p[start], p[start + 1], p[start + 2]]
}

fn render_name_table(
    ppu: &mut NesPPU,
    frame: &mut Frame,
    name_table: &[u8],
    view_port: Rect,
    shift_x: isize,
    shift_y: isize,
) {
    let bank = ppu.ctrl.background_pattern_addr();
    let attribute_table = &name_table[0x03C0..0x0400];

    for i in 0..0x03C0 {
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile_idx = name_table[i] as u16;
        let mut tile: [u8; 15] = [0; 15];
        let mut i = 0;
        for addr in (bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize {
            tile[i] = ppu.mapper.chr_rom(addr);
            i += 1
        }

        let palette = bg_pallette(ppu, attribute_table, tile_column, tile_row);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[palette[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("can't be"),
                };

                let pixel_x = tile_column * 8 + x;
                let pixel_y = tile_row * 8 + y;
                if pixel_x >= view_port.x1
                    && pixel_x < view_port.x2
                    && pixel_y >= view_port.y1
                    && pixel_y < view_port.y2
                {
                    frame.set_pixel(
                        (shift_x + pixel_x as isize) as usize,
                        (shift_y + pixel_y as isize) as usize,
                        rgb,
                    )
                }
            }
        }
    }
}
