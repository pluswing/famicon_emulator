use sdl2::{
    audio::{AudioCallback, AudioDevice, AudioSpecDesired},
    AudioSubsystem,
};
use std::sync::mpsc;
use std::time::Duration;

pub struct NesApu {
    channel1_sender: mpsc::Sender<SquareNote>,
    channel1_device: AudioDevice<SquareWave>,

    channel2_sender: mpsc::Sender<SquareNote>,
    channel2_device: AudioDevice<SquareWave>,

    channel3_sender: mpsc::Sender<TriangleNote>,
    channel3_device: AudioDevice<TriangleWave>,

    channel4_sender: mpsc::Sender<NoiseNote>,
    channel4_device: AudioDevice<Noise>,
}

impl NesApu {
    pub fn new(sdl: &sdl2::Sdl) -> Result<NesApu, String> {
        let audio_subsystem = sdl.audio().unwrap();

        let (mut channel1_sender, channel1_device) = init_square(&audio_subsystem);
        let (mut channel2_sender, channel2_device) = init_square(&audio_subsystem);
        let (mut channel3_sender, channel3_device) = init_triangle(&audio_subsystem);
        let (mut channel4_sender, channel4_device) = init_noise(&audio_subsystem);

        Ok(NesApu {
            channel1_sender: channel1_sender,
            channel1_device: channel1_device,
            channel2_sender: channel2_sender,
            channel2_device: channel2_device,
            channel3_sender: channel3_sender,
            channel3_device: channel3_device,
            channel4_sender: channel4_sender,
            channel4_device: channel4_device,
        })
    }

    pub fn set1ch(&mut self, note: SquareNote) {
        let res = self.channel1_sender.send(note);
        match res {
            Ok(_) => {}
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    pub fn set2ch(&mut self, note: SquareNote) {
        let res = self.channel2_sender.send(note);
        match res {
            Ok(_) => {}
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
    pub fn set3ch(&mut self, note: TriangleNote) {
        let res = self.channel3_sender.send(note);
        match res {
            Ok(_) => {}
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
    pub fn set4ch(&mut self, note: NoiseNote) {
        let res = self.channel4_sender.send(note);
        match res {
            Ok(_) => {}
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}

#[derive(Debug)]
pub struct SquareNote {
    pub hz: f32,     // 音程
    pub duty: f32,   // デューティー比 0.125|0.25|0.5|0.75
    pub volume: f32, // ボリューム
}
struct SquareWave {
    note: SquareNote,
    freq: f32,
    phase: f32,
    receiver: mpsc::Receiver<SquareNote>,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            match self.receiver.recv_timeout(Duration::from_nanos(0)) {
                Ok(n) => {
                    self.note = n;
                }
                _ => {}
            }
            if self.note.volume == 0.0 {
                *x = 0.0
            } else {
                *x = if self.phase <= self.note.duty {
                    self.note.volume
                } else {
                    -self.note.volume
                };
            }
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

fn init_square(subsystem: &AudioSubsystem) -> (mpsc::Sender<SquareNote>, AudioDevice<SquareWave>) {
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let (sender, receiver) = mpsc::channel();
    let dev = subsystem
        .open_playback(None, &desired_spec, |spec| SquareWave {
            note: SquareNote {
                hz: 0.0,
                volume: 0.0,
                duty: 0.0,
            },
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
        })
        .unwrap();
    dev.resume();
    return (sender, dev);
}

#[derive(Debug)]
pub struct TriangleNote {
    pub hz: f32, // 音程
}

struct TriangleWave {
    note: TriangleNote,
    freq: f32,
    phase: f32,
    receiver: mpsc::Receiver<TriangleNote>,
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            match self.receiver.recv_timeout(Duration::from_nanos(0)) {
                Ok(n) => {
                    self.note = n;
                }
                _ => {}
            }
            *x = if self.phase < 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } * 4.0;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0
        }
    }
}

fn init_triangle(
    subsystem: &AudioSubsystem,
) -> (mpsc::Sender<TriangleNote>, AudioDevice<TriangleWave>) {
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: Some(2),  // default sample size
    };

    let (sender, receiver) = mpsc::channel();
    let dev = subsystem
        .open_playback(None, &desired_spec, |spec| TriangleWave {
            note: TriangleNote { hz: 0.0 },
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
        })
        .unwrap();

    dev.resume();
    return (sender, dev);
}

struct NoiseRandom {
    value: u16,
    freq: u16,
    counter: u16,
}

impl NoiseRandom {
    fn newLong() -> NoiseRandom {
        NoiseRandom {
            value: 0x4000,
            freq: 32767,
            counter: 0,
        }
    }
    fn newShort() -> NoiseRandom {
        NoiseRandom {
            value: 0x4000,
            freq: 93,
            counter: 0,
        }
    }
    fn next(&mut self) -> bool {
        let x = ((self.value & 0x02) >> 1) ^ (self.value & 0x01);
        self.value = self.value >> 1;
        self.value = self.value | (x << 14);
        let r = self.value & 0x01 == 1;

        self.counter += 1;
        if self.counter >= self.freq {
            self.counter = 0;
            self.value = 0x4000;
        }
        r
    }
}

#[derive(Debug)]
pub struct NoiseNote {
    pub is_long: bool,
    pub volume: f32,
    pub div: u16,
}

struct Noise {
    note: NoiseNote,
    random: NoiseRandom,
    freq: f32,
    phase: f32,
    table: Vec<f32>,
    receiver: mpsc::Receiver<NoiseNote>,
}

impl AudioCallback for Noise {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let cpu_clock = 1789772.5;

        for x in out.iter_mut() {
            match self.receiver.recv_timeout(Duration::from_nanos(0)) {
                Ok(n) => {
                    self.random = if n.is_long {
                        NoiseRandom::newLong()
                    } else {
                        NoiseRandom::newShort()
                    };
                    self.note = n;
                    self.phase = 0.0;

                    if self.note.div != 0 {
                        let mut v = false;
                        self.table = (0..cpu_clock as i32)
                            .map(|i| {
                                if i % self.note.div as i32 == 0 {
                                    v = self.random.next();
                                }
                                return if v { 1.0 } else { 0.0 } * self.note.volume;
                            })
                            .collect();
                    }
                }
                _ => {}
            }
            if self.note.div == 0 {
                *x = 0.0;
                continue;
            } else {
                *x = self.table[(self.phase * cpu_clock) as usize];
                self.phase = (self.phase + 1.0 / self.freq) % 1.0
            }
        }
    }
}

fn init_noise(subsystem: &AudioSubsystem) -> (mpsc::Sender<NoiseNote>, AudioDevice<Noise>) {
    // noise spec
    // 周波数は、ファミコンのCPUのクロック周波数(1789772.5)を、除数で割った値。
    // 以下が除数テーブル。（設定値から、0x80を引いてテーブル参照する。）
    // let div_table = vec![
    //     4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    // ];
    // see https://gameprogrammingunit.web.fc2.com/al/noise.htm
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let (sender, receiver) = mpsc::channel();
    let dev = subsystem
        .open_playback(None, &desired_spec, |spec| Noise {
            note: NoiseNote {
                is_long: true,
                volume: 1.0,
                div: 0,
            },
            random: NoiseRandom::newLong(),
            freq: spec.freq as f32,
            phase: 0.0,
            table: vec![],
            receiver: receiver,
        })
        .unwrap();

    dev.resume();

    return (sender, dev);
}
