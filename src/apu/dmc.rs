use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

use super::ChannelEvent;

static FREQUENCY_TABLE: [u8; 16] = [
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
    ByteCount(u8),
    Frequency(u8),
    Enable(bool),
    Reset(),
}

pub struct DmcWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<DmcEvent>,
    sender: Sender<ChannelEvent>,
    enabled: bool,

    byte_count: u16,
}

impl AudioCallback for DmcWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            loop {
                let res = self.receiver.recv_timeout(Duration::from_millis(0));
                match res {
                    Ok(DmcEvent::Enable(b)) => self.enabled = b,
                    Ok(DmcEvent::ByteCount(b)) => {
                        // FIXME なんかちがう？ %LLLL.LLLL0001 = (L * 16) + 1 バイト
                        self.byte_count = (((b as u16) << 4) + 1) << 3;
                    }
                    Ok(DmcEvent::Frequency(f)) => {}
                    Ok(DmcEvent::Reset()) => {}
                    Err(_) => break,
                }
            }

            /*

            WaveCh5SampleCounter => byte_count
            WaveCh5FrequencyData => FREQUENCY_TABLE[frequency_index]
            tmpWaveBaseCount2 => phase
            WaveCh5Register => 波形データが入る
            WaveCh5SampleAddress => start_addr
            (tmpIO2[0x10] & 0x40) => loop_flag
            tmpIO2[0x10] & 0x80 => irq_enable
            WaveCh5DeltaCounter => delta_counter


              if(this.WaveCh5SampleCounter !== 0) {
                    angle = (tmpWaveBaseCount2 / this.WaveCh5FrequencyData[tmpIO2[0x10] & 0x0F]) & 0x1F;

                    // if(this.WaveCh5Angle !== angle) {
                    //     var ii = this.WaveCh5Angle;
                    //     var jj = 0;
                    //     if(ii !== -1) {
                    //         jj = angle;
                    //         if(jj < ii)
                    //             jj += 32;
                    //     }
                    //     this.WaveCh5Angle = angle;

                        for(; ii<jj; ii++){
                            if((this.WaveCh5SampleCounter & 0x0007) === 0) {
                                if(this.WaveCh5SampleCounter !== 0){
                                    this.WaveCh5Register = this.ROM[(this.WaveCh5SampleAddress >> 13) + 2][this.WaveCh5SampleAddress & 0x1FFF];
                                    this.WaveCh5SampleAddress++;
                                    this.CPUClock += 4;
                                }
                            }

                            if(this.WaveCh5SampleCounter !== 0) {
                                if((this.WaveCh5Register & 0x01) === 0x00) {
                                    if(this.WaveCh5DeltaCounter > 1)
                                        this.WaveCh5DeltaCounter -= 2;
                                } else {
                                    if(this.WaveCh5DeltaCounter < 126)
                                        this.WaveCh5DeltaCounter += 2;
                                }
                                this.WaveCh5Register >>= 1;
                                this.WaveCh5SampleCounter--;
                            }
                        }
                    }

                    if(this.WaveCh5SampleCounter === 0) {
                        if((tmpIO2[0x10] & 0x40) === 0x40)
                            this.SetCh5Delta();
                        else
                            this.toIRQ |= tmpIO2[0x10] & 0x80;
                    }
                }
                return (all_out + this.WaveCh5DeltaCounter) << 5;
                         */

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
