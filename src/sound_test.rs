use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver};
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

// ノイズ
struct NoiseRandom {
    // ランダム生成モードフラグがセットされていればショートモード、 クリアされていればロングモードとなります。 ショートモードの時のビットシーケンスは93ビット、 ロングモードの時は32767ビットです。
    bit: u8,
    cycle: u32,
    cycle_counter: u32,

    value: u16,
}

impl NoiseRandom {
    pub fn new() -> Self {
        NoiseRandom {
            bit: 1,
            cycle: 32767,
            cycle_counter: 0,
            value: 1,
        }
    }

    pub fn next(&mut self) -> bool {
        // 15ビットシフトレジスタにはリセット時に1をセットしておく必要があります。 タイマによってシフトレジスタが励起されるたびに1ビット右シフトし、 ビット14には、ショートモード時にはビット0とビット6のEORを、 ロングモード時にはビット0とビット1のEORを入れます。
        if self.cycle_counter >= self.cycle {
            self.value = 1;
        }
        self.cycle_counter += 1;

        // ロングモード時にはビット0とビット1のEORを入れます。
        let b = (self.value & 0x01) ^ ((self.value >> self.bit) & 0x01);
        self.value = self.value >> 1;
        self.value = self.value & 0b01_1111_1111_1111 | b << 14;

        // シフトレジスタのビット0が1なら、チャンネルの出力は0となります。
        self.value & 0x01 != 0
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
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
                hz: 261.626,
                volume: 0.25,
                duty: 0.25,
            },
        })
        .unwrap();

    device.resume();

    std::thread::sleep(Duration::from_millis(2000));

    sender
        .send(SquareNote {
            hz: 293.665,
            volume: 0.25,
            duty: 0.125,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(2000));

    sender
        .send(SquareNote {
            hz: 329.628,
            volume: 0.25,
            duty: 0.125,
        })
        .unwrap();

    std::thread::sleep(Duration::from_millis(2000));
}
