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
