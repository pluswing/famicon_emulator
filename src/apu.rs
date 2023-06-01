pub struct NesAPU {
    ch1_register: Ch1Register,

    ch1_device: AudioDevice<SquareWave>,
    ch1_sender: Sender<SquareNote>,
}

const NES_CPU_CLOCK: f32 = 1_789_773.0; // 1.78MHz

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender) = init_square(&sdl_context);

        NesAPU {
            ch1_register: Ch1Register::new(),

            ch1_device: ch1_device,
            ch1_sender: ch1_sender,
        }
    }

    pub fn write1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);

        let duty = match self.ch1_register.duty() {
            0 => 0.125,
            1 => 0.25,
            2 => 0.50,
            3 => 0.75,
            _ => panic!(
                "can't be {} {:02X}",
                self.ch1_register.duty(),
                self.ch1_register.tone_volume,
            ),
        };

        let volume = (self.ch1_register.volume() as f32) / 15.0;

        let hz = NES_CPU_CLOCK / (16.0 * (self.ch1_register.hz() as f32 + 1.0));

        self.ch1_sender
            .send(SquareNote {
                hz: hz,
                volume: volume,
                duty: duty,
            })
            .unwrap();
    }
}

struct Ch1Register {
    pub tone_volume: u8,
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
        (self.hz_high_key_on as u16 & 0x07 << 8) | (self.hz_low as u16)
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

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
struct SquareNote {
    hz: f32,
    volume: f32,
    duty: f32,
}

struct SquareWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<SquareNote>,
    note: SquareNote,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(note) => self.note = note,
                Err(_) => {}
            }
            *x = if self.phase <= self.note.duty {
                self.note.volume
            } else {
                -self.note.volume
            };
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareNote>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<SquareNote>();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SquareWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            note: SquareNote {
                hz: 0.0,
                volume: 0.0,
                duty: 0.0,
            },
        })
        .unwrap();

    device.resume();

    (device, sender)
}
