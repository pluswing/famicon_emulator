use std::thread;
use std::time::Duration;
use std::{sync::mpsc::Receiver, thread::sleep};

use midir::{Ignore, MidiInput, MidiOutput, MidiOutputConnection};
use once_cell::sync::Lazy;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SoundEvent {
    pub channel: u8,
    pub hz: f32,

    pub duty: u8,
    pub volume: f32,
    pub is_long: bool,
}

struct Key {
    y: u8,
    x: u8,
}

const INPUT_CLIENT_NAME: &str = "launchpad.rs input";
const OUTPUT_CLINET_NAME: &str = "launchpad.rs output";

const LAUNCHPAD_MINI_MK3: &str = "Launchpad Mini MK3 LPMiniMK3 MIDI In";
const LAUNCHPAD_X: &str = "Launchpad X LPX MIDI In";

const EVENT_TYPE_PAD: u8 = 144; // 背景白の鍵盤ボタンイベント
const EVENT_TYPE_CONTROL: u8 = 176; // 背景黒の上1行目と右1列のボタンイベント

static mut mode: u8 = 1;

static HZ_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    vec![
        // C4 ~ B4
        261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994, 391.995, 415.305, 440.000,
        466.164, 493.883,
    ]
});
static KEY_TABLE: Lazy<Vec<Key>> = Lazy::new(|| {
    vec![
        Key { y: 0, x: 1 }, // ド
        Key { y: 1, x: 2 }, // ド#
        Key { y: 0, x: 2 }, // レ
        Key { y: 1, x: 3 }, // レ#
        Key { y: 0, x: 3 }, // ミ
        Key { y: 0, x: 4 }, // ファ
        Key { y: 1, x: 5 }, // ファ#
        Key { y: 0, x: 5 }, // ソ
        Key { y: 1, x: 6 }, // ソ#
        Key { y: 0, x: 6 }, // ラ
        Key { y: 1, x: 7 }, // ラ#
        Key { y: 0, x: 7 }, // シ
    ]
});

static DUTY_TABLE: Lazy<Vec<f32>> = Lazy::new(|| vec![0.125, 0.25, 0.5]);

static OCTAVE_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    vec![
        0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0,
    ]
});
static NOISE_TABLE: Lazy<Vec<i32>> = Lazy::new(|| {
    vec![
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ]
});

pub fn start_launchpad(
    ch1: Receiver<SoundEvent>,
    ch2: Receiver<SoundEvent>,
    ch3: Receiver<SoundEvent>,
    ch4: Receiver<SoundEvent>,
) {
    thread::spawn(move || loop {
        let (mut out, input) = connect();

        let (s, r) = std::sync::mpsc::channel::<Vec<u8>>();

        let in_ports = input.ports();
        let _conn_in = input
            .connect(
                in_ports.get(1).unwrap(),
                INPUT_CLIENT_NAME,
                move |_, message, _| {
                    let mut m = Vec::new();
                    for i in message.iter() {
                        m.push(*i)
                    }
                    s.send(m).unwrap();
                },
                (),
            )
            .unwrap();

        let mut ev = SoundEvent {
            channel: 0,
            hz: 0.0,
            duty: 0,
            volume: 0.0,
            is_long: false,
        };
        loop {
            loop {
                let e = r.recv_timeout(Duration::from_millis(0));
                match e {
                    Ok(m) => {
                        if m[0] == EVENT_TYPE_CONTROL {
                            let note = m[1];
                            let y = note / 10;
                            let x = note - (y * 10);

                            set_color(&mut out, 1, 9, 5);
                            set_color(&mut out, 2, 9, 5);
                            set_color(&mut out, 3, 9, 5);
                            set_color(&mut out, 4, 9, 5);

                            if x == 1 && y == 9 {
                                set_pulse(&mut out, 1, 9, 5);
                                unsafe {
                                    mode = 1;
                                }
                            }
                            if x == 2 && y == 9 {
                                set_pulse(&mut out, 2, 9, 5);
                                unsafe {
                                    mode = 2;
                                }
                            }
                            if x == 3 && y == 9 {
                                set_pulse(&mut out, 3, 9, 5);
                                unsafe {
                                    mode = 3;
                                }
                            }
                            if x == 4 && y == 9 {
                                set_pulse(&mut out, 4, 9, 5);
                                unsafe {
                                    mode = 4;
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
            loop {
                let e = ch1.recv_timeout(Duration::from_millis(0));
                match e {
                    Ok(e) => {
                        if unsafe { mode == 1 } {
                            ev = e
                        }
                    }
                    Err(_) => break,
                }
            }
            loop {
                let e = ch2.recv_timeout(Duration::from_millis(0));
                match e {
                    Ok(e) => {
                        if unsafe { mode == 2 } {
                            ev = e
                        }
                    }
                    Err(_) => break,
                }
            }
            loop {
                let e = ch3.recv_timeout(Duration::from_millis(0));
                match e {
                    Ok(e) => {
                        if unsafe { mode == 3 } {
                            ev = e
                        }
                    }
                    Err(_) => break,
                }
            }
            loop {
                let e = ch4.recv_timeout(Duration::from_millis(0));
                match e {
                    Ok(e) => {
                        if unsafe { mode == 4 } {
                            ev = e
                        }
                    }
                    Err(_) => break,
                }
            }

            loop {
                if ev.channel == unsafe { mode } {
                    match ev.channel {
                        1 | 2 | 3 => {
                            if ev.hz == 0.0 {
                                break;
                            }
                            let mut octave = 0;
                            loop {
                                let max = HZ_TABLE[11] + ((HZ_TABLE[0] * 2.0) - HZ_TABLE[11]) / 2.0;
                                let omax = max * OCTAVE_TABLE[octave];
                                if ev.hz <= omax {
                                    break;
                                }
                                octave += 1;
                            }
                            let hz = ev.hz / OCTAVE_TABLE[octave];
                            let mut index = 0;
                            let mut min = f32::MAX;

                            for (i, &value) in HZ_TABLE.iter().enumerate() {
                                let distance = (value - hz).abs();
                                if distance < min {
                                    min = distance;
                                    index = i;
                                }
                            }
                            set_current(&mut out, octave, index, ev.channel);
                            break;
                        }
                        4 => break,
                        _ => break,
                    }
                } else {
                    break;
                }
            }

            sleep(Duration::from_millis(1))
        }
        println!("!!exit loop")
    });
}

fn clear_key(output: &mut MidiOutputConnection) {
    let c = 53;
    let b = 0;
    let w = 37;

    set_color(output, 1, 1, c);
    set_color(output, 1, 3, c);
    set_color(output, 1, 5, c);
    set_color(output, 1, 7, c);

    set_color(output, 8, 1, c);
    set_color(output, 8, 3, c);
    set_color(output, 8, 5, c);
    set_color(output, 8, 7, c);

    for y in [1, 3, 5, 7] {
        for x in 2..=7 {
            set_color(output, x, y, w);
        }
    }

    for y in [2, 4, 6, 8] {
        for x in [2, 3, 5, 6, 7] {
            set_color(output, x, y, w);
        }
    }

    for y in [2, 4, 6, 8] {
        for x in [1, 4, 8] {
            set_color(output, x, y, b);
        }
    }
}

static mut last: Lazy<Vec<usize>> = Lazy::new(|| vec![0, 0]);

fn set_current(output: &mut MidiOutputConnection, octave: usize, index: usize, channel: u8) {
    if (unsafe { last[0] } == octave) && (unsafe { last[1] } == index) {
        return;
    }
    unsafe {
        last[0] = octave;
        last[1] = index;
    }

    clear_key(output);
    let m = if channel == 3 { 1 } else { 3 };
    let y: isize = (octave as isize - m) * 2 - 1;
    if y < 0 {
        return;
    }
    let k = &KEY_TABLE[index];
    set_color(output, k.x, y as u8 + k.y, 5);
}

fn connect() -> (MidiOutputConnection, MidiInput) {
    let mut midi_in = MidiInput::new(INPUT_CLIENT_NAME).unwrap();
    midi_in.ignore(Ignore::None);

    let mut midi_out;
    let mut port_index: i32 = -1;
    let mut launchpad_name: &str = "";
    loop {
        midi_out = MidiOutput::new(OUTPUT_CLINET_NAME).unwrap();
        if midi_out.ports().len() > 0 {
            for (i, p) in midi_out.ports().iter().enumerate() {
                let name = midi_out.port_name(p).unwrap();
                if name == LAUNCHPAD_MINI_MK3 {
                    port_index = i as i32;
                    launchpad_name = LAUNCHPAD_MINI_MK3
                }
                if name == LAUNCHPAD_X {
                    port_index = i as i32;
                    launchpad_name = LAUNCHPAD_X
                }
            }
            if port_index != -1 {
                break;
            }
        }
        sleep(Duration::from_millis(1 * 1000));
    }
    let out_ports = midi_out.ports();
    let mut _conn_out = midi_out
        .connect(
            out_ports.get(port_index as usize).unwrap(),
            OUTPUT_CLINET_NAME,
        )
        .unwrap();

    println!("connected. launchpadName = {}", launchpad_name);
    println!("set programmer mode");
    set_programmer_mode(launchpad_name, &mut _conn_out).unwrap();
    set_pulse(&mut _conn_out, 9, 9, 5);

    set_pulse(&mut _conn_out, 1, 9, 5);
    set_color(&mut _conn_out, 2, 9, 5);
    set_color(&mut _conn_out, 3, 9, 5);
    set_color(&mut _conn_out, 4, 9, 5);

    return (_conn_out, midi_in);
}

fn set_programmer_mode(
    launchpad_name: &str,
    output: &mut MidiOutputConnection,
) -> Result<(), midir::SendError> {
    match launchpad_name {
        LAUNCHPAD_X => output.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0C, 0x00, 0x7F, 0xF7]),
        LAUNCHPAD_MINI_MK3 => output.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x00, 0x7F, 0xF7]),
        _ => Ok(()),
    }
}

fn set_pulse(output: &mut MidiOutputConnection, x: u8, y: u8, index_color: u8) {
    let note = x + y * 10;
    output.send(&[0x92, note, index_color]);
}

fn set_color(output: &mut MidiOutputConnection, x: u8, y: u8, index_color: u8) {
    let note = x + y * 10;
    output.send(&[0x90, note, index_color]);
}
