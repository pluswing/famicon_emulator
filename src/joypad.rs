use bitflags::bitflags;

bitflags! {
  pub struct JoypadButton: u8 {
    const RIGHT      = 0b1000_0000;
    const LEFT       = 0b0100_0000;
    const DOWN       = 0b0010_0000;
    const UP         = 0b0001_0000;
    const START      = 0b0000_1000;
    const SELECT     = 0b0000_0100;
    const BUTTON_B   = 0b0000_0010;
    const BUTTON_A   = 0b0000_0001;
  }
}

pub struct Joypad {
    strobe: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.button_index = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 {
            return 1;
        }

        let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
        if !self.strobe && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }

    // TODO joypad.set_button_pressed_status
}
