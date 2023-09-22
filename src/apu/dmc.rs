use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use super::ChannelEvent;

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
    Enable(bool),
    Reset(),
}

pub struct DmcWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<DmcEvent>,
    sender: Sender<ChannelEvent>,
    enabled: bool,
}

impl AudioCallback for DmcWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(DmcEvent::Enable(b)) => self.enabled = b,
                    Ok(DmcEvent::Reset()) => {}
                    Err(_) => break,
                }
            }
            // TODO
            *x = 0.0;
        }
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
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| DmcWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            sender: sender2,
            enabled: true,
        })
        .unwrap();

    device.resume();

    (device, sender, receiver2)
}
