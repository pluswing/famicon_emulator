mod apu_for_launchpad;

use self::apu_for_launchpad::{NesApu, NoiseNote, SquareNote, TriangleNote};

use midir::{Ignore, MidiInput, MidiOutput, MidiOutputConnection};
use once_cell::sync::Lazy;
use std::error::Error;
use std::process::exit;
use std::sync::{mpsc, Mutex};
use std::thread::sleep;
use std::time::Duration;

const LAUNCHPAD_MINI_MK3: &str = "Launchpad Mini MK3 LPMiniMK3 MIDI In";
const LAUNCHPAD_X: &str = "Launchpad X LPX MIDI In";

const COLOR_OFF: u8 = 0;
const COLOR_RED: u8 = 5;
const COLOR_GREEN: u8 = 21;
const COLOR_YELLOW: u8 = 13;
const COLOR_BLUE: u8 = 37;

const EVENT_TYPE_PAD: u8 = 144; // 背景白の鍵盤ボタンイベント
const EVENT_TYPE_CONTROL: u8 = 176; // 背景黒の上1行目と右1列のボタンイベント

const MODE_1CH: i32 = 1;
const MODE_3CH: i32 = 3;
const MODE_4CH: i32 = 4;

static mut mode: i32 = MODE_1CH;
static mut launchpadName: &str = "";

fn main() -> Result<(), String> {
    runMidi();
    Ok(())
}

fn runMidi() -> Result<(), Box<dyn Error>> {
    let input_client_name = "launchpad.rs input";
    let output_clinet_name = "launchpad.rs output";

    let mut midi_in = MidiInput::new(input_client_name)?;
    midi_in.ignore(Ignore::None);

    let mut midi_out;
    let mut port_index: i32 = -1;
    loop {
        midi_out = MidiOutput::new(output_clinet_name)?;
        if midi_out.ports().len() > 0 {
            for (i, p) in midi_out.ports().iter().enumerate() {
                let name = midi_out.port_name(p)?;
                if name == LAUNCHPAD_MINI_MK3 {
                    port_index = i as i32;
                    unsafe { launchpadName = LAUNCHPAD_MINI_MK3 }
                }
                if name == LAUNCHPAD_X {
                    port_index = i as i32;
                    unsafe { launchpadName = LAUNCHPAD_X }
                }
            }
            if port_index != -1 {
                break;
            }
        }
        println!("launchpad not found. wait connect...");
        sleep(Duration::from_millis(1 * 1000));
    }

    println!("launchpad connected!\nexit to Ctrl + C.");
    let out_ports = midi_out.ports();
    let mut _conn_out = midi_out.connect(
        out_ports.get(port_index as usize).unwrap(),
        output_clinet_name,
    )?;

    let sdl = sdl2::init()?;
    let mut apu = NesApu::new(&sdl).unwrap();

    let (sender1, receiver1) = mpsc::channel();
    sender1.send(vec![EVENT_TYPE_CONTROL, 0, 0]).unwrap();
    let in_ports = midi_in.ports();
    let _conn_in = midi_in.connect(
        in_ports.get(1).unwrap(),
        input_client_name,
        move |_, message, _| {
            let mut m = Vec::new();
            for i in message.iter() {
                m.push(*i)
            }
            sender1.send(m).unwrap();
        },
        (),
    );

    set_programmer_mode(&mut _conn_out);
    set_pulse(&mut _conn_out, 9, 9, 5);

    let (sender2, receiver2) = mpsc::channel();
    ctrlc::set_handler(move || {
        sender2.send(vec![0]);
    })?;

    loop {
        let res = receiver1.recv_timeout(Duration::from_millis(0));
        match res {
            Ok(message) => handle_root(&mut apu, &mut _conn_out, &message),
            _ => {
                let res2 = receiver2.recv_timeout(Duration::from_millis(0));
                match res2 {
                    Ok(_) => exit(0),
                    _ => {}
                }
            }
        }
    }
}

struct PulseChannelState {
    volume: usize,
    duty: usize,
    octave: usize,
    push_count: u8,
}
struct TriangleChannelState {
    octave: usize,
    push_count: u8,
}

struct NoiseChannelState {
    volume: usize,
    push_count: u8,
    is_long: bool,
}

static HZ_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    vec![
        // C4 ~ B4
        261.626, 277.183, 293.665, 311.127, 329.628, 349.228, 369.994, 391.995, 415.305, 440.000,
        466.164, 493.883,
    ]
});
static DUTY_TABLE: Lazy<Vec<f32>> = Lazy::new(|| vec![0.125, 0.25, 0.5]);
static VOLUME_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    vec![
        1.0 * 1.0 / 12.0,
        1.0 * 2.0 / 12.0,
        1.0 * 3.0 / 12.0,
        1.0 * 4.0 / 12.0,
        1.0 * 5.0 / 12.0,
        1.0 * 6.0 / 12.0,
        1.0 * 7.0 / 12.0,
        1.0 * 8.0 / 12.0,
        1.0 * 9.0 / 12.0,
        1.0 * 10.0 / 12.0,
        1.0 * 11.0 / 12.0,
        1.0 * 12.0 / 12.0,
    ]
});
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
static state1ch: Lazy<Mutex<PulseChannelState>> = Lazy::new(|| {
    Mutex::new(PulseChannelState {
        volume: 6,
        duty: 2,
        octave: 3,
        push_count: 0,
    })
});
static state3ch: Lazy<Mutex<TriangleChannelState>> = Lazy::new(|| {
    Mutex::new(TriangleChannelState {
        octave: 3,
        push_count: 0,
    })
});
static state4ch: Lazy<Mutex<NoiseChannelState>> = Lazy::new(|| {
    Mutex::new(NoiseChannelState {
        volume: 6,
        push_count: 0,
        is_long: true,
    })
});

fn set_programmer_mode(output: &mut MidiOutputConnection) -> Result<(), midir::SendError> {
    unsafe {
        match launchpadName {
            LAUNCHPAD_X => output.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0C, 0x00, 0x7F, 0xF7]),
            LAUNCHPAD_MINI_MK3 => {
                output.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x00, 0x7F, 0xF7])
            }
            _ => Ok(()),
        }
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

fn handle_root(apu: &mut NesApu, output: &mut MidiOutputConnection, message: &Vec<u8>) {
    // note押下以外は無視
    if message[0] != EVENT_TYPE_PAD && message[0] != EVENT_TYPE_CONTROL {
        return;
    }

    let note = message[1];
    let y = note / 10;
    let x = note - (y * 10);

    unsafe {
        if x == 6 && y == 9 {
            mode = MODE_4CH
        }
        if x == 7 && y == 9 {
            mode = MODE_1CH
        }
        if x == 8 && y == 9 {
            mode = MODE_3CH
        }

        match mode {
            MODE_4CH => {
                set_pulse(output, 6, 9, COLOR_YELLOW);
                set_color(output, 7, 9, COLOR_RED);
                set_color(output, 8, 9, COLOR_GREEN);
                handle_launchpad_only_4ch_mode_basic(apu, output, message);
            }
            MODE_1CH => {
                set_color(output, 6, 9, COLOR_YELLOW);
                set_pulse(output, 7, 9, COLOR_RED);
                set_color(output, 8, 9, COLOR_GREEN);
                handle_launchpad_only_1ch_mode_basic(apu, output, message);
            }
            MODE_3CH => {
                set_color(output, 6, 9, COLOR_YELLOW);
                set_color(output, 7, 9, COLOR_RED);
                set_pulse(output, 8, 9, COLOR_GREEN);
                handle_launchpad_only_3ch_mode_basic(apu, output, message);
            }
            _ => {}
        }
    }
}

fn handle_launchpad_only_1ch_mode_basic(
    apu: &mut NesApu,
    output: &mut MidiOutputConnection,
    message: &Vec<u8>,
) {
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

    set_color(output, 1, 9, COLOR_RED); // octave
    set_color(output, 2, 9, COLOR_RED);
    set_color(output, 3, 9, COLOR_YELLOW); // volume
    set_color(output, 4, 9, COLOR_YELLOW);
    set_color(output, 5, 9, COLOR_BLUE); // duty

    let pad_octave_count = 4;

    if state1ch.lock().unwrap().octave == OCTAVE_TABLE.len() - 1 - pad_octave_count {
        set_color(output, 1, 9, COLOR_OFF);
    }
    if state1ch.lock().unwrap().octave == 0 {
        set_color(output, 2, 9, COLOR_OFF);
    }
    if state1ch.lock().unwrap().volume == 0 {
        set_color(output, 3, 9, COLOR_OFF);
    }
    if state1ch.lock().unwrap().volume == VOLUME_TABLE.len() - 1 {
        set_color(output, 4, 9, COLOR_OFF);
    }

    if message[0] != EVENT_TYPE_PAD && message[0] != EVENT_TYPE_CONTROL {
        // note押下以外は無視
        return;
    }

    let note = message[1];
    let y = note / 10;
    let x = note - (y * 10);
    let velocity = message[2];
    let push = velocity != 0;

    if y >= 9 {
        if !push {
            return;
        }
        match x {
            1 => {
                if state1ch.lock().unwrap().octave < OCTAVE_TABLE.len() - 1 - pad_octave_count {
                    state1ch.lock().unwrap().octave += 1
                }
            }
            2 => {
                if state1ch.lock().unwrap().octave > 0 {
                    state1ch.lock().unwrap().octave -= 1
                }
            }
            3 => {
                if state1ch.lock().unwrap().volume > 0 {
                    state1ch.lock().unwrap().volume -= 1
                }
            }
            4 => {
                if state1ch.lock().unwrap().volume < VOLUME_TABLE.len() - 1 {
                    state1ch.lock().unwrap().volume += 1
                }
            }
            5 => {
                state1ch.lock().unwrap().duty += 1;
                if state1ch.lock().unwrap().duty >= DUTY_TABLE.len() {
                    state1ch.lock().unwrap().duty = 0
                }
            }
            _ => {}
        }
        return;
    }

    if x >= 9 {
        return;
    }

    handle1ch_full_basic(apu, x, y, push)
}

fn handle_launchpad_only_3ch_mode_basic(
    apu: &mut NesApu,
    output: &mut MidiOutputConnection,
    message: &Vec<u8>,
) {
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

    set_color(output, 1, 9, COLOR_RED); // octave
    set_color(output, 2, 9, COLOR_RED);
    set_color(output, 3, 9, COLOR_OFF);
    set_color(output, 4, 9, COLOR_OFF);
    set_color(output, 5, 9, COLOR_OFF);

    let pad_octave_count = 4;

    if state3ch.lock().unwrap().octave == OCTAVE_TABLE.len() - 1 - pad_octave_count {
        set_color(output, 1, 9, COLOR_OFF);
    }
    if state3ch.lock().unwrap().octave == 0 {
        set_color(output, 2, 9, COLOR_OFF);
    }

    if message[0] != EVENT_TYPE_PAD && message[0] != EVENT_TYPE_CONTROL {
        // note押下以外は無視
        return;
    }

    let note = message[1];
    let y = note / 10;
    let x = note - (y * 10);
    let velocity = message[2];
    let push = velocity != 0;

    if y >= 9 {
        if !push {
            return;
        }
        match x {
            1 => {
                if state3ch.lock().unwrap().octave < OCTAVE_TABLE.len() - 1 - pad_octave_count {
                    state3ch.lock().unwrap().octave += 1
                }
            }
            2 => {
                if state3ch.lock().unwrap().octave > 0 {
                    state3ch.lock().unwrap().octave -= 1
                }
            }
            _ => {}
        }
        return;
    }
    if x >= 9 {
        return;
    }

    handle3ch_full_basic(apu, x, y, push)
}

fn handle_launchpad_only_4ch_mode_basic(
    apu: &mut NesApu,
    output: &mut MidiOutputConnection,
    message: &Vec<u8>,
) {
    // set base color

    for x in [1, 2, 3, 4] {
        for y in [1, 2, 3, 4] {
            set_color(output, x, y, 17);
        }
    }

    for x in [5, 6, 7, 8] {
        for y in [1, 2, 3, 4] {
            set_color(output, x, y, 9);
        }
    }

    for x in 1..=8 {
        for y in [5, 6, 7, 8] {
            set_color(output, x, y, 0);
        }
    }

    set_color(output, 1, 9, COLOR_OFF);
    set_color(output, 2, 9, COLOR_OFF);
    set_color(output, 3, 9, COLOR_YELLOW); // volume
    set_color(output, 4, 9, COLOR_YELLOW);
    set_color(output, 5, 9, COLOR_OFF);

    if state4ch.lock().unwrap().volume == 0 {
        set_color(output, 3, 9, COLOR_OFF);
    }
    if state4ch.lock().unwrap().volume == VOLUME_TABLE.len() - 1 {
        set_color(output, 4, 9, COLOR_OFF);
    }

    if message[0] != EVENT_TYPE_PAD && message[0] != EVENT_TYPE_CONTROL {
        // note押下以外は無視
        return;
    }

    let note = message[1];
    let y = note / 10;
    let x = note - (y * 10);
    let velocity = message[2];
    let push = velocity != 0;

    if y >= 9 {
        if !push {
            return;
        }
        match x {
            3 => {
                if state4ch.lock().unwrap().volume > 0 {
                    state4ch.lock().unwrap().volume -= 1
                }
            }
            4 => {
                if state4ch.lock().unwrap().volume < VOLUME_TABLE.len() - 1 {
                    state4ch.lock().unwrap().volume += 1
                }
            }
            _ => {}
        }
        return;
    }

    if x >= 9 {
        return;
    }

    handle4ch_full_basic(apu, x, y, push)
}

fn handle1ch_full_basic(apu: &mut NesApu, x: u8, y: u8, push: bool) {
    if !push {
        if state1ch.lock().unwrap().push_count <= 0 {
            return;
        }
        state1ch.lock().unwrap().push_count -= 1;
        if state1ch.lock().unwrap().push_count == 0 {
            apu.set1ch(SquareNote {
                hz: 0.0,
                duty: 0.0,
                volume: 0.0,
            });
        }
        return;
    }

    let uk: [i32; 8] = [-1, 1, 3, -1, 6, 8, 10, -1];
    let dk: [i32; 8] = [0, 2, 4, 5, 7, 9, 11, 0];

    let index = if y % 2 == 0 {
        uk[(x - 1) as usize]
    } else {
        dk[(x - 1) as usize]
    };

    if index < 0 {
        return;
    }

    let octave = (((y - 1) / 2) + if x == 8 { 2 } else { 1 }) as usize;
    state1ch.lock().unwrap().push_count += 1;
    let o = OCTAVE_TABLE[octave + state1ch.lock().unwrap().octave];
    let v = VOLUME_TABLE[state1ch.lock().unwrap().volume];
    let d = DUTY_TABLE[state1ch.lock().unwrap().duty];
    apu.set1ch(SquareNote {
        hz: HZ_TABLE[index as usize] * o,
        duty: d,
        volume: v,
    });
}

fn handle3ch_full_basic(apu: &mut NesApu, x: u8, y: u8, push: bool) {
    if !push {
        if state3ch.lock().unwrap().push_count <= 0 {
            return;
        }
        state3ch.lock().unwrap().push_count -= 1;
        if state3ch.lock().unwrap().push_count == 0 {
            apu.set3ch(TriangleNote { hz: 0.0 });
        }
        return;
    }

    let uk: [i32; 8] = [-1, 1, 3, -1, 6, 8, 10, -1];
    let dk: [i32; 8] = [0, 2, 4, 5, 7, 9, 11, 0];

    let index = if y % 2 == 0 {
        uk[(x - 1) as usize]
    } else {
        dk[(x - 1) as usize]
    };

    if index < 0 {
        return;
    }

    let octave = (((y - 1) / 2) + if x == 8 { 2 } else { 1 }) as usize;
    state3ch.lock().unwrap().push_count += 1;
    let o = OCTAVE_TABLE[octave + state3ch.lock().unwrap().octave];
    apu.set3ch(TriangleNote {
        hz: HZ_TABLE[index as usize] * o,
    });
}

fn handle4ch_full_basic(apu: &mut NesApu, x: u8, y: u8, push: bool) {
    if y > 4 {
        return;
    }

    if !push {
        if state4ch.lock().unwrap().push_count <= 0 {
            return;
        }
        state4ch.lock().unwrap().push_count -= 1;
        if state4ch.lock().unwrap().push_count == 0 {
            apu.set4ch(NoiseNote {
                is_long: false,
                div: 0,
                volume: 0.0,
            });
        }
        return;
    }

    state4ch.lock().unwrap().push_count += 1;
    let index = ((if x > 4 { x - 4 } else { x } + 4 * (y - 1)) - 1) as usize;
    let d = NOISE_TABLE[index];
    apu.set4ch(NoiseNote {
        is_long: x > 4,
        div: d as u16,
        volume: VOLUME_TABLE[state4ch.lock().unwrap().volume],
    });
}

// chrome mode ------------------------------------------------------------

fn handle_launchpad_only_3ch_mode_chrom(
    apu: &mut NesApu,
    output: &mut MidiOutputConnection,
    message: &Vec<u8>,
) {
    // set base color

    let x = 0;
    let y = 0;

    let c = 9;
    let b = 0;
    let w = 3;

    set_color(output, x + 1, y + 1, c);
    set_color(output, x + 2, y + 1, b);
    set_color(output, x + 3, y + 1, w);
    set_color(output, x + 4, y + 1, b);
    set_color(output, x + 5, y + 1, w);
    set_color(output, x + 1, y + 2, w);
    set_color(output, x + 2, y + 2, b);
    set_color(output, x + 3, y + 2, w);
    set_color(output, x + 4, y + 2, b);
    set_color(output, x + 5, y + 2, w);
    set_color(output, x + 1, y + 3, b);
    set_color(output, x + 2, y + 3, w);

    set_color(output, x + 3, y + 3, c);
    set_color(output, x + 4, y + 3, b);
    set_color(output, x + 5, y + 3, w);
    set_color(output, x + 1, y + 4, b);
    set_color(output, x + 2, y + 4, w);
    set_color(output, x + 3, y + 4, w);
    set_color(output, x + 4, y + 4, b);
    set_color(output, x + 5, y + 4, w);
    set_color(output, x + 1, y + 5, b);
    set_color(output, x + 2, y + 5, w);
    set_color(output, x + 3, y + 5, b);
    set_color(output, x + 4, y + 5, w);

    set_color(output, x + 5, y + 5, c);
    set_color(output, x + 1, y + 6, b);
    set_color(output, x + 2, y + 6, w);
    set_color(output, x + 3, y + 6, b);
    set_color(output, x + 4, y + 6, w);
    set_color(output, x + 5, y + 6, w);
    set_color(output, x + 1, y + 7, b);
    set_color(output, x + 2, y + 7, w);
    set_color(output, x + 3, y + 7, b);
    set_color(output, x + 4, y + 7, w);
    set_color(output, x + 5, y + 7, b);
    set_color(output, x + 1, y + 8, w);

    set_color(output, x + 2, y + 8, c);
    set_color(output, x + 3, y + 8, b);
    set_color(output, x + 4, y + 8, w);
    set_color(output, x + 5, y + 8, b);

    // right
    set_color(output, x + 6, y + 1, w);
    set_color(output, x + 7, y + 1, b);
    set_color(output, x + 8, y + 1, w);

    set_color(output, x + 6, y + 2, b);
    set_color(output, x + 7, y + 2, w);
    set_color(output, x + 8, y + 2, c);

    set_color(output, x + 6, y + 3, b);
    set_color(output, x + 7, y + 3, w);
    set_color(output, x + 8, y + 3, w);

    set_color(output, x + 6, y + 4, b);
    set_color(output, x + 7, y + 4, w);
    set_color(output, x + 8, y + 4, b);

    set_color(output, x + 6, y + 5, b);
    set_color(output, x + 7, y + 5, w);
    set_color(output, x + 8, y + 5, b);

    set_color(output, x + 6, y + 6, b);
    set_color(output, x + 7, y + 6, w);
    set_color(output, x + 8, y + 6, b);

    set_color(output, x + 6, y + 7, w);
    set_color(output, x + 7, y + 7, c);
    set_color(output, x + 8, y + 7, b);

    set_color(output, x + 6, y + 8, w);
    set_color(output, x + 7, y + 8, w);
    set_color(output, x + 8, y + 8, b);

    if message[0] != EVENT_TYPE_PAD && message[0] != EVENT_TYPE_CONTROL {
        // note押下以外は無視
        return;
    }
    let note = message[1];
    let y = note / 10;
    let x = note - (y * 10);
    let velocity = message[2];
    let push = velocity != 0;

    if x == 9 || y == 9 {
        return;
    }
    handle3ch_full_chrome(apu, x, y, push)

    // if x <= 4 && y <= 4 {
    //     // 左下
    //     handle4ch(apu, x, y, push)
    // } else if x <= 4 && y > 4 {
    //     // 左上
    //     handle1ch(apu, x, y - 4, push)
    // } else if x > 4 && y <= 4 {
    //     // 右下
    //     handle3ch(apu, x - 4, y, push)
    // } else if x > 4 && y > 4 {
    //     // 右上
    //     handle2ch(apu, x - 4, y - 4, push)
    // }
}

fn handle3ch_full_chrome(apu: &mut NesApu, x: u8, y: u8, push: bool) {
    if !push {
        state3ch.lock().unwrap().push_count -= 1;
        if state3ch.lock().unwrap().push_count == 0 {
            apu.set3ch(TriangleNote { hz: 0.0 });
        }
        return;
    }
    let index = ((y - 1) * 5 + (x - 1)) as usize;
    let octave = (index / 12) + 1 as usize;
    let i = index % 12 as usize;
    state3ch.lock().unwrap().push_count += 1;
    let o = OCTAVE_TABLE[octave];
    apu.set3ch(TriangleNote {
        hz: HZ_TABLE[i] * o,
    });
}
