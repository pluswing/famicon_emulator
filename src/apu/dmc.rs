use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use crate::MAPPER;

use super::{ChannelEvent, MASTER_VOLUME, NES_CPU_CLOCK};

static FREQUENCY_TABLE: [u16; 16] = [
    0x1AC, 0x17C, 0x154, 0x140, 0x11E, 0x0FE, 0x0E2, 0x0D6, 0x0BE, 0x0A0, 0x08E, 0x080, 0x06A,
    0x054, 0x048, 0x036,
];

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

#[derive(Debug, Clone, PartialEq)]
pub enum DmcEvent {
    IrqEnable(bool),
    Loop(bool),
    Frequency(u8),
    Delta(u8),
    StartAddr(u8),
    ByteCount(u8),

    Enable(bool),
    Reset(),
}

pub struct DmcWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<DmcEvent>,
    sender: Sender<ChannelEvent>,
    enabled: bool,

    irq_enabled: bool,
    loop_flag: bool,
    frequency_index: u8,
    delta_counter: u8,
    start_addr: u8,
    byte_count: u8,

    data: u8,
    frequency: f32,
    sample_addr: u16,
    counter: u32,
}

impl AudioCallback for DmcWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(DmcEvent::Enable(b)) => self.enabled = b,
                    Ok(DmcEvent::IrqEnable(b)) => {
                        self.irq_enabled = b;
                    }
                    Ok(DmcEvent::Loop(b)) => {
                        self.loop_flag = b;
                    }
                    Ok(DmcEvent::Frequency(f)) => {
                        self.frequency_index = f;
                        self.frequency = NES_CPU_CLOCK / FREQUENCY_TABLE[f as usize] as f32;
                    }
                    Ok(DmcEvent::Delta(d)) => {
                        self.delta_counter = d;
                        // 仮置き
                        self.byte_count = 1;
                        self.counter = (1 * 8) as u32 * 0x10 + 1;
                    }
                    Ok(DmcEvent::StartAddr(s)) => {
                        self.start_addr = s;
                        self.sample_addr = s as u16 * 0x40 + 0xC000
                    }
                    Ok(DmcEvent::ByteCount(b)) => {
                        self.byte_count = b;
                        self.counter = (b * 8) as u32 * 0x10 + 1;
                    }
                    Ok(DmcEvent::Reset()) => {}
                    Err(_) => break,
                }
            }

            let last_phase = self.phase;
            self.phase = (self.phase + self.frequency / self.freq) % 1.0;

            if last_phase > self.phase {
                if self.counter == 0 {
                    *x = 0.0;
                    continue;
                }
                if self.counter & 0x0007 == 0 {
                    if self.counter != 0 {
                        unsafe {
                            self.data = MAPPER.read_prg_rom(self.sample_addr);
                        };
                        if self.sample_addr == 0xFFFF {
                            self.sample_addr = 0x8000;
                        } else {
                            self.sample_addr += 1;
                        }
                    }
                }

                if self.counter != 0 {
                    if self.data & 0x01 == 0x00 {
                        if self.delta_counter > 1 {
                            self.delta_counter -= 2
                        }
                    } else {
                        if self.delta_counter < 126 {
                            self.delta_counter += 2
                        }
                    }
                    self.data = self.data >> 1;
                    self.counter -= 1;
                    self.sender
                        .send(ChannelEvent::LengthCounter(self.counter))
                        .unwrap();
                }

                if self.counter == 0 {
                    if self.loop_flag {
                        self.set_delta();
                    } else {
                        if self.irq_enabled {
                            // TODO IRQを発生させる
                        }
                    }
                }
            }
            if self.delta_counter == 0 || self.counter == 0 {
                *x = 0.0;
            } else {
                *x = ((self.delta_counter as f32 - 64.0) / 64.0) * MASTER_VOLUME;
            }

            if !self.enabled {
                *x = 0.0;
            }
        }
    }
}

impl DmcWave {
    fn set_delta(&mut self) {
        self.delta_counter = self.delta_counter;
        self.sample_addr = self.start_addr as u16 * 0x40 + 0xC000;
        self.counter = (self.byte_count * 8) as u32 * 0x10 + 1;
        self.data = 0;
        // self.toIRQ &= ~0x80;
    }
}

pub fn init_dmc(
    sdl_context: &sdl2::Sdl,
) -> (
    AudioDevice<DmcWave>,
    Sender<DmcEvent>,
    Receiver<ChannelEvent>,
) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<DmcEvent>();
    let (sender2, receiver2) = channel::<ChannelEvent>();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2),
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| DmcWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            enabled: true,
            irq_enabled: false,
            loop_flag: false,
            frequency_index: 0,
            delta_counter: 0,
            start_addr: 0,
            byte_count: 1,
            data: 0,
            frequency: NES_CPU_CLOCK / FREQUENCY_TABLE[0] as f32,
            sample_addr: 0xC000,
            counter: (0 * 8) as u32 * 0x10 + 1,
        })
        .unwrap();

    device.resume();

    (device, sender, receiver2)
}
