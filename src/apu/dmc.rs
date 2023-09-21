pub struct Ch5Register {
    // 4010
    pub irq_enabled: bool,
    pub loop_flag: bool,
    pub frequency_index: u8,

    // 4011
    pub delta_counter: u8,

    // 4012
    pub start_addr: u8,

    // 4013
    pub byte_count: u8,
}

impl Ch5Register {
    pub fn new() -> Self {
        Ch5Register {
            irq_enabled: false,
            loop_flag: false,
            frequency_index: 0,
            delta_counter: 0,
            start_addr: 0,
            byte_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4010 => {
                self.irq_enabled = value & 0x80 != 0;
                self.loop_flag = value & 0x40 != 0;
                self.frequency_index = value & 0x0F;
            }
            0x4011 => {
                self.delta_counter = value & 0x7F;
            }
            0x4012 => {
                self.start_addr = value;
            }
            0x4013 => {
                self.byte_count = value;
            }
            _ => panic!("can't be"),
        }
    }
}
