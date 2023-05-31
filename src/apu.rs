pub struct NesAPU {
    ch1_register: Ch1Register,
}

impl NesAPU {
    pub fn new() -> Self {
        NesAPU {
            ch1_register: Ch1Register::new(),
        }
    }

    pub fn write1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);
    }
}

struct Ch1Register {
    tone_volume: u8,
    sweep: u8,
    hz_low: u8,
    hz_high_key_on: u8,
}

impl Ch1Register {
    pub fn new() -> Self {
        Ch1Register {
            tone_volume: 0x00,
            sweep: 0x00,
            hz_low: 0x00,
            hz_high_key_on: 0x00,
        }
    }

    pub fn duty(&self) -> u8 {
        // 00：12.5%　01：25%　10：50%　11：75%
        (self.tone_volume & 0xC0) >> 6
    }

    pub fn volume(&self) -> u8 {
        // （0で消音、15が最大）
        self.tone_volume & 0x0F
    }

    pub fn hz(&self) -> u16 {
        self.hz_high_key_on as u16 & 0x07 << 8 & self.hz_low as u16
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => {
                self.tone_volume = value;
            }
            0x4001 => {
                self.sweep = value;
            }
            0x4002 => {
                self.hz_low = value;
            }
            0x4003 => {
                self.hz_high_key_on = value;
            }
            _ => panic!("can't be"),
        }
    }
}
