use log::{debug, info};

use crate::frame::Frame;
use crate::ppu::NesPPU;
use crate::rom::Mirroring;
use crate::{palette, MAPPER};

const SCREEN_W: usize = 256;
const SCREEN_H: usize = 240;

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

pub fn render(ppu: &NesPPU, frame: &mut Frame, scanline: usize) {
    // 描画範囲
    let draw_rect = Rect::new(0, scanline - 8, SCREEN_W, scanline);

    draw_background(ppu, frame, &draw_rect);
    draw_sprites(ppu, frame, &draw_rect);
}

fn draw_background(ppu: &NesPPU, frame: &mut Frame, draw_rect: &Rect) {
    let scroll_x = (ppu.scroll.scroll_x) as usize;
    let scroll_y = (ppu.scroll.scroll_y) as usize;
    let mirroring = unsafe { MAPPER.mirroring() };
    let vram_a = &ppu.vram[0x000..0x400];
    let vram_b = &ppu.vram[0x400..0x800];
    let (top_left, top_right, bottom_left, bottom_right) =
        match (&mirroring, ppu.ctrl.nametable_addr()) {
            (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) => {
                (vram_a, vram_b, vram_a, vram_b)
            }
            (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) => {
                (vram_b, vram_a, vram_b, vram_a)
            }
            (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => {
                (vram_a, vram_a, vram_b, vram_b)
            }
            (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => {
                (vram_b, vram_b, vram_a, vram_a)
            }
            (_, _) => {
                panic!("Not supported mirroring type {:?}", mirroring);
            }
        };

    // 左上
    render_name_table(
        ppu,
        frame,
        top_left,
        Rect::new(scroll_x, scroll_y, SCREEN_W, SCREEN_H),
        -(scroll_x as isize),
        -(scroll_y as isize),
        &draw_rect,
    );

    // 右上
    render_name_table(
        ppu,
        frame,
        top_right,
        Rect::new(0, scroll_y, scroll_x, SCREEN_H),
        (SCREEN_W - scroll_x) as isize,
        -(scroll_y as isize),
        &draw_rect,
    );

    // 左下
    render_name_table(
        ppu,
        frame,
        bottom_left,
        Rect::new(scroll_x, 0, SCREEN_W, scroll_y),
        -(scroll_x as isize),
        (SCREEN_H - scroll_y) as isize,
        &draw_rect,
    );

    // 右下
    render_name_table(
        ppu,
        frame,
        bottom_right,
        Rect::new(0, 0, scroll_x, scroll_y),
        (SCREEN_W - scroll_x) as isize,
        (SCREEN_H - scroll_y) as isize,
        &draw_rect,
    );
}

fn draw_sprites(ppu: &NesPPU, frame: &mut Frame, draw_rect: &Rect) {
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_y = ppu.oam_data[i] as usize;
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let attr = ppu.oam_data[i + 2];
        let tile_x = ppu.oam_data[i + 3] as usize;

        if ppu.ctrl.is_sprite_8x16_mode() {
            let bank = if (tile_idx & 0x01) == 0 { 0 } else { 0x1000 };
            let top = bank + (tile_idx & 0xFE) * 16;
            let bottom = bank + ((tile_idx & 0xFE) + 1) * 16;

            let flip_vertical = (attr >> 7 & 1) == 1;
            if flip_vertical {
                draw_tile(ppu, frame, bottom, tile_x, tile_y, attr, draw_rect);
                draw_tile(ppu, frame, top, tile_x, tile_y + 8, attr, draw_rect);
            } else {
                draw_tile(ppu, frame, top, tile_x, tile_y, attr, draw_rect);
                draw_tile(ppu, frame, bottom, tile_x, tile_y + 8, attr, draw_rect);
            }
        } else {
            let bank: u16 = ppu.ctrl.sprite_pattern_addr();
            let start = bank + tile_idx * 16;
            draw_tile(ppu, frame, start, tile_x, tile_y, attr, draw_rect);
        }
    }
}

fn draw_tile(
    ppu: &NesPPU,
    frame: &mut Frame,
    start: u16,
    tile_x: usize,
    tile_y: usize,
    attr: u8,
    draw_rect: &Rect,
) {
    let flip_vertical = (attr >> 7 & 1) == 1;
    let flip_horizontal = (attr >> 6 & 1) == 1;
    let palette_idx = attr & 0b11;
    let sprite_palette = sprite_palette(ppu, tile_y, palette_idx);

    let mut tile: [u8; 16] = [0; 16];
    for i in 0..=15 {
        tile[i] = unsafe { MAPPER.read_chr_rom(start + i as u16) }
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

            let (_x, _y) = match (flip_horizontal, flip_vertical) {
                (false, false) => (tile_x + x, tile_y + y),
                (true, false) => (tile_x + 7 - x, tile_y + y),
                (false, true) => (tile_x + x, tile_y + 7 - y),
                (true, true) => (tile_x + 7 - x, tile_y + 7 - y),
            };

            if _x >= draw_rect.x1 && _x < draw_rect.x2 && _y >= draw_rect.y1 && _y < draw_rect.y2 {
                frame.set_pixel(_x, _y, rgb)
            }
        }
    }
}

fn bg_pallette(
    ppu: &NesPPU,
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
        p[0] & 0x3F,
        p[pallette_start] & 0x3F,
        p[pallette_start + 1] & 0x3F,
        p[pallette_start + 2] & 0x3F,
    ]
}

fn sprite_palette(ppu: &NesPPU, tile_y: usize, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + (palette_idx * 4) as usize;
    let p = ppu.read_palette_table(tile_y);
    [0, p[start] & 0x3F, p[start + 1] & 0x3F, p[start + 2] & 0x3F]
}

fn render_name_table(
    ppu: &NesPPU,
    frame: &mut Frame,
    name_table: &[u8],
    view_port: Rect,
    shift_x: isize,
    shift_y: isize,
    draw_rect: &Rect,
) {
    let bank = ppu.ctrl.background_pattern_addr();
    let attribute_table = &name_table[0x03C0..0x0400];

    for i in 0..0x03C0 {
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile_idx = name_table[i] as u16;

        let start = bank + tile_idx * 16;
        let mut tile: [u8; 16] = [0; 16];
        for i in 0..=15 {
            tile[i] = unsafe { MAPPER.read_chr_rom(start + i as u16) }
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
                    let x = (shift_x + pixel_x as isize) as usize;
                    let y = (shift_y + pixel_y as isize) as usize;
                    if x >= draw_rect.x1
                        && x < draw_rect.x2
                        && y >= draw_rect.y1
                        && y < draw_rect.y2
                    {
                        frame.set_pixel(x, y, rgb)
                    }
                }
            }
        }
    }
}
