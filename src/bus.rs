pub struct Bus {
    cpu_vram: [u8; 2048],
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048],
        }
    }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl Mem for Bus {
    pub fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0010_0000_0000_0111;
                todo!("PPU is not supported yet")
            }
            _ => {
                println!("Ignoreing mem access at {}", addr);
                0
            }
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0010_0000_0000_0111;
                todo!("PPU is not supported yet")
            }
            _ => {
                println!("Ignoreing mem write-access at {}", addr);
                0
            }
        }
    }

    // TODO
    // mem_read_u16
    // mem_write_u16
}
