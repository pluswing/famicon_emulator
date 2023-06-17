use log::info;

pub struct NesAPU {
    ch1_register: Ch1Register,
    ch2_register: Ch2Register,
    ch3_register: Ch3Register,
    ch4_register: Ch4Register,

    ch1_device: AudioDevice<SquareWave>,
    ch1_sender: Sender<SquareEvent>,

    ch2_device: AudioDevice<SquareWave>,
    ch2_sender: Sender<SquareEvent>,

    ch3_device: AudioDevice<TriangleWave>,
    ch3_sender: Sender<TriangleEvent>,

    ch4_device: AudioDevice<NoiseWave>,
    ch4_sender: Sender<NoiseEvent>,

    // see: https://www.nesdev.org/wiki/APU_Frame_Counter
    frame_counter: u8,
    cycles: usize,

    // see: https://www.nesdev.org/apu_ref.txt
    status: u8,
}

const NES_CPU_CLOCK: f32 = 1_789_772.5; // 1.78MHz

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let (ch1_device, ch1_sender) = init_square(&sdl_context);
        let (ch2_device, ch2_sender) = init_square(&sdl_context);
        let (ch3_device, ch3_sender) = init_triangle(&sdl_context);
        let (ch4_device, ch4_sender) = init_noise(&sdl_context);

        NesAPU {
            ch1_register: Ch1Register::new(),
            ch2_register: Ch2Register::new(),
            ch3_register: Ch3Register::new(),
            ch4_register: Ch4Register::new(),

            ch1_device: ch1_device,
            ch1_sender: ch1_sender,

            ch2_device: ch2_device,
            ch2_sender: ch2_sender,

            ch3_device: ch3_device,
            ch3_sender: ch3_sender,

            ch4_device: ch4_device,
            ch4_sender: ch4_sender,

            frame_counter: 0x40, // 割り込み禁止がデフォルト
            cycles: 0,
            status: 0,
        }
    }

    pub fn write1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);

        let duty = match self.ch1_register.duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.50,
            0x03 => 0.75,
            _ => panic!("can't be"),
        };

        let volume = (self.ch1_register.volume as f32) / 15.0;

        let hz = NES_CPU_CLOCK / (16.0 * (self.ch1_register.frequency as f32 + 1.0));

        match addr {
            0x4000 => {
                self.ch1_sender
                    .send(SquareEvent::Envelope(Envelope::new(
                        self.ch1_register.volume,
                        self.ch1_register.envelope_flag,
                    )))
                    .unwrap();
            }

            0x4001 => {
                self.ch1_sender
                    .send(SquareEvent::Sweep(Sweep::new(
                        self.ch1_register.sweep_change_amount,
                        self.ch1_register.sweep_direction,
                        self.ch1_register.sweep_timer_count,
                        self.ch1_register.sweep_enabled != 0,
                    )))
                    .unwrap();
            }

            0x4003 => {
                // change keyoff
                self.ch1_sender
                    .send(SquareEvent::KeyOff(KeyOff::new(
                        self.ch1_register.key_off_counter_flag,
                        KEYOFF_TABLE[self.ch1_register.key_off_count as usize],
                    )))
                    .unwrap();

                self.ch1_sender
                    .send(SquareEvent::Note(SquareNote {
                        hz: hz,
                        volume: volume,
                        duty: duty,
                    }))
                    .unwrap();
            }
            _ => {}
        }
    }

    pub fn write2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);

        let duty = match self.ch2_register.duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.50,
            0x03 => 0.75,
            _ => panic!("can't be",),
        };

        let volume = (self.ch2_register.volume as f32) / 15.0;

        let hz = NES_CPU_CLOCK / (16.0 * (self.ch2_register.frequency as f32 + 1.0));

        match addr {
            0x4004 => {
                self.ch2_sender
                    .send(SquareEvent::Envelope(Envelope::new(
                        self.ch2_register.volume,
                        self.ch2_register.envelope_flag,
                    )))
                    .unwrap();
            }

            0x4005 => {
                self.ch2_sender
                    .send(SquareEvent::Sweep(Sweep::new(
                        self.ch2_register.sweep_change_amount,
                        self.ch2_register.sweep_direction,
                        self.ch2_register.sweep_timer_count,
                        self.ch2_register.sweep_enabled != 0,
                    )))
                    .unwrap();
            }

            0x4007 => {
                // change keyoff
                self.ch2_sender
                    .send(SquareEvent::KeyOff(KeyOff::new(
                        self.ch2_register.key_off_counter_flag,
                        KEYOFF_TABLE[self.ch2_register.key_off_count as usize],
                    )))
                    .unwrap();

                self.ch2_sender
                    .send(SquareEvent::Note(SquareNote {
                        hz: hz,
                        volume: volume,
                        duty: duty,
                    }))
                    .unwrap();
            }
            _ => {}
        }
    }

    pub fn write3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);

        let hz = NES_CPU_CLOCK / (32.0 * (self.ch3_register.frequency as f32 + 1.0));

        match addr {
            0x400B => {
                self.ch3_sender
                    .send(TriangleEvent::KeyOff(KeyOff::new(
                        self.ch3_register.key_off_counter_flag,
                        KEYOFF_TABLE[self.ch3_register.key_off_count as usize],
                    )))
                    .unwrap();

                self.ch3_sender
                    .send(TriangleEvent::Note(TriangleNote { hz }))
                    .unwrap();
            }
            _ => {}
        }
    }

    pub fn write4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);

        let hz = NES_CPU_CLOCK / NOIZE_TABLE[self.ch4_register.frequency as usize] as f32;
        let is_long = self.ch4_register.kind == 0;
        let volume = (self.ch4_register.volume as f32) / 15.0;

        match addr {
            0x400C => {
                self.ch4_sender
                    .send(NoiseEvent::Envelope(Envelope::new(
                        self.ch4_register.volume,
                        self.ch4_register.envelope_flag,
                    )))
                    .unwrap();
            }
            0x400F => {
                self.ch4_sender
                    .send(NoiseEvent::KeyOff(KeyOff::new(
                        self.ch4_register.key_off_counter_flag,
                        KEYOFF_TABLE[self.ch4_register.key_off_count as usize],
                    )))
                    .unwrap();

                self.ch4_sender
                    .send(NoiseEvent::Note(NoiseNote {
                        hz: hz,
                        is_long: is_long,
                        volume: volume,
                    }))
                    .unwrap();
            }
            _ => {}
        }
    }

    pub fn write_status(&mut self, value: u8) {
        self.status = value;
    }
    pub fn read_status(&mut self) -> u8 {
        self.frame_counter = self.frame_counter & !0x40;
        self.status = self.status | 0x1F;
        self.status
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        self.frame_counter = value;
    }

    pub fn apu_irq_enabled(&self) -> bool {
        return (self.frame_counter & 0x40) == 0;
    }

    pub fn apu_frame_count_interval(&self) -> usize {
        if (self.frame_counter & 0x80) == 0 {
            4
        } else {
            5
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.status = self.status & !0x40;
        if !self.apu_irq_enabled() {
            self.cycles = 0;
            return false;
        }
        self.cycles += cycles as usize;
        // 7457を掛けたものが、インターバルになる。
        // see: https://pgate1.at-ninja.jp/NES_on_FPGA/nes_apu.htm#frame
        let fci = self.apu_frame_count_interval();
        if fci == 5 {
            // ==5の場合、IRQは発生しない
            self.cycles = 0;
            return false;
        }
        let interval = 7457 * 4; // 60Hzで割り込みが発生する
        while self.cycles >= interval {
            self.cycles -= interval;
            self.status = self.status | 0x40;
            return true;
        }
        return false;
    }
}

struct Ch1Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    sweep_change_amount: u8,
    sweep_direction: u8,
    sweep_timer_count: u8,
    sweep_enabled: u8,

    frequency: u16,

    key_off_count: u8,
}

impl Ch1Register {
    pub fn new() -> Self {
        Ch1Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            duty: 0,

            sweep_change_amount: 0,
            sweep_direction: 0,
            sweep_timer_count: 0,
            sweep_enabled: 0,

            frequency: 0,

            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4000 => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
                self.duty = (value & 0xC0) >> 6;
            }
            0x4001 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_direction = (value & 0x08) >> 3;
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enabled = (value & 0x80) >> 7;
            }
            0x4002 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4003 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("can't be"),
        }
    }
}

struct Ch2Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    sweep_change_amount: u8,
    sweep_direction: u8,
    sweep_timer_count: u8,
    sweep_enabled: u8,

    frequency: u16,

    key_off_count: u8,
}

impl Ch2Register {
    pub fn new() -> Self {
        Ch2Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            duty: 0,

            sweep_change_amount: 0,
            sweep_direction: 0,
            sweep_timer_count: 0,
            sweep_enabled: 0,

            frequency: 0,

            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4004 => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
                self.duty = (value & 0xC0) >> 6;
            }
            0x4005 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_direction = (value & 0x08) >> 3;
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enabled = (value & 0x80) >> 7;
            }
            0x4006 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4007 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("can't be"),
        }
    }
}

struct Ch3Register {
    // 4008
    length: u8,
    key_off_counter_flag: bool,

    // 400A, 400B
    frequency: u16,
    key_off_count: u8,
}

impl Ch3Register {
    pub fn new() -> Self {
        Ch3Register {
            length: 0,
            key_off_counter_flag: false,
            frequency: 0,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4008 => {
                self.length = value & 0x7F;
                self.key_off_counter_flag = (value & 0x80) == 0;
            }
            0x4009 => {}
            0x400A => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x400B => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("can't be"),
        }
    }
}

struct Ch4Register {
    // 400C
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,

    // 400E
    frequency: u8,
    kind: u8,

    // 400F
    key_off_count: u8,
}

impl Ch4Register {
    pub fn new() -> Self {
        Ch4Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            frequency: 0,
            kind: 0,
            key_off_count: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x400C => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
            }
            0x400E => {
                self.frequency = value & 0x0F;
                self.kind = value & 0x80 >> 7;
            }
            0x400F => {
                self.key_off_count = (value & 0xF8) >> 3;
            }
            _ => panic!("can't be"),
        }
    }
}

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
struct FrameSequencer {
    counter: usize,
    interval: usize,
}

impl FrameSequencer {
    fn new(interval: usize) -> Self {
        FrameSequencer {
            counter: 0,
            interval,
        }
    }
    fn next(&mut self) -> bool {
        self.counter += 1;
        if self.counter >= self.interval {
            self.counter = 0;
            return true;
        }
        return false;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Envelope {
    interval: u8,
    counter: u8,
    volume: u8,
    loop_flag: bool,
    sequencer: FrameSequencer,
}

impl Envelope {
    fn new(rate: u8, loop_flag: bool) -> Self {
        Envelope {
            interval: rate,
            counter: 0,
            volume: 0x0F,
            loop_flag: loop_flag,
            sequencer: FrameSequencer::new(44100 / 240),
        }
    }

    fn reset(&mut self) {
        self.counter = 0;
        self.volume = 0x0F;
    }

    fn next(&mut self) -> f32 {
        if self.interval == 0 {
            return 0.0;
        }
        if self.volume == 0 {
            return 0.0;
        }
        if !self.sequencer.next() {
            return self.volume as f32 / 15.0;
        }
        self.counter += 1;
        if self.counter >= self.interval {
            self.volume -= 1;
        }
        let v = self.volume;
        if self.volume <= 0 {
            if self.loop_flag {
                self.reset()
            }
        }
        v as f32 / 15.0
    }
}

#[derive(Debug, Clone, PartialEq)]
struct KeyOff {
    enable: bool,
    count: u8,
    counter: u8,
    sequencer: FrameSequencer,
}

impl KeyOff {
    fn new(enable: bool, count: u8) -> Self {
        KeyOff {
            enable,
            count,
            counter: 0,
            sequencer: FrameSequencer::new(44100 / 120),
        }
    }

    fn next(&mut self) -> bool {
        if !self.enable {
            return false;
        }
        if self.count < self.counter {
            return true;
        }
        if !self.sequencer.next() {
            return false;
        }
        self.counter += 1;
        return false;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Sweep {
    change_amount: u8,
    direction: u8,
    count: u8,
    enable: bool,
    counter: u8,
    value: f32,
    sequencer: FrameSequencer,
}

impl Sweep {
    fn new(change_amount: u8, direction: u8, count: u8, enable: bool) -> Self {
        Sweep {
            change_amount,
            direction,
            count,
            enable,
            counter: 0,
            value: 1.0,
            sequencer: FrameSequencer::new(44100 / 120),
        }
    }

    fn is_add(&self) -> bool {
        self.direction != 0
    }

    fn next(&mut self) -> f32 {
        if !self.enable {
            return 1.0;
        }
        if self.count < self.counter {
            self.value *= 2.0;
        }
        if !self.sequencer.next() {
            return self.value;
        }
        self.counter += 1;
        return self.value;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SquareNote {
    hz: f32,
    volume: f32,
    duty: f32,
}

enum SquareEvent {
    Note(SquareNote),
    Envelope(Envelope),
    KeyOff(KeyOff),
    Sweep(Sweep),
}

struct SquareWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<SquareEvent>,
    envelope: Envelope,
    keyoff: KeyOff,
    sweep: Sweep,
    note: SquareNote,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(SquareEvent::Note(note)) => self.note = note,
                Ok(SquareEvent::Envelope(e)) => self.envelope = e,
                Ok(SquareEvent::KeyOff(k)) => self.keyoff = k,
                Ok(SquareEvent::Sweep(s)) => self.sweep = s,
                Err(_) => {}
            }
            let mut volume = self.envelope.next();
            if self.keyoff.next() {
                volume = 0.0;
            }

            let v = self.sweep.next();
            let hz = if self.sweep.is_add() {
                self.note.hz + self.note.hz * v
            } else {
                self.note.hz - self.note.hz * v
            };

            *x = if self.phase <= self.note.duty {
                volume
            } else {
                -volume
            };
            self.phase = (self.phase + hz / self.freq) % 1.0;
        }
    }
}

fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<SquareEvent>();

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
            envelope: Envelope::new(1, false),
            keyoff: KeyOff::new(false, 0),
            sweep: Sweep::new(0, 0, 0, false),
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

#[derive(Debug, Clone, PartialEq)]
struct TriangleNote {
    hz: f32,
}

enum TriangleEvent {
    Note(TriangleNote),
    KeyOff(KeyOff),
}

struct TriangleWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<TriangleEvent>,
    note: TriangleNote,
    keyoff: KeyOff,
}

impl AudioCallback for TriangleWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(TriangleEvent::Note(note)) => self.note = note,
                Ok(TriangleEvent::KeyOff(k)) => self.keyoff = k,
                Err(_) => {}
            }
            let mut volume = 4.0;
            if self.keyoff.next() {
                volume = 0.0;
            }
            *x = (if self.phase <= 0.5 {
                self.phase
            } else {
                1.0 - self.phase
            } - 0.25)
                * volume;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
        }
    }
}

fn init_triangle(sdl_context: &sdl2::Sdl) -> (AudioDevice<TriangleWave>, Sender<TriangleEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<TriangleEvent>();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| TriangleWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            note: TriangleNote { hz: 0.0 },
            keyoff: KeyOff::new(false, 0),
        })
        .unwrap();

    device.resume();

    (device, sender)
}

lazy_static! {
    pub static ref NOIZE_TABLE: Vec<u16> = vec![
        0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0030, 0x0040, 0x0050, 0x0065, 0x007F, 0x00BE,
        0x00FE, 0x017D, 0x01FC, 0x03F9, 0x07F2,
    ];
}

lazy_static! {
    pub static ref KEYOFF_TABLE: Vec<u8> = vec![
        0x05, 0x7F, 0x0A, 0x01, 0x14, 0x02, 0x28, 0x03, 0x50, 0x04, 0x1E, 0x05, 0x07, 0x06, 0x0D,
        0x07, 0x06, 0x08, 0x0C, 0x09, 0x18, 0x0A, 0x30, 0x0B, 0x60, 0x0C, 0x24, 0x0D, 0x08, 0x0E,
        0x10, 0x0F,
    ];
}

#[derive(Debug, Clone, PartialEq)]
struct NoiseNote {
    hz: f32,
    is_long: bool,
    volume: f32,
}

enum NoiseEvent {
    Note(NoiseNote),
    Envelope(Envelope),
    KeyOff(KeyOff),
}

struct NoiseWave {
    freq: f32,
    phase: f32,
    receiver: Receiver<NoiseEvent>,
    value: bool,
    long_random: NoiseRandom,
    short_random: NoiseRandom,

    note: NoiseNote,
    envelope: Envelope,
    keyoff: KeyOff,
}

impl AudioCallback for NoiseWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [Self::Channel]) {
        for x in out.iter_mut() {
            let res = self.receiver.recv_timeout(Duration::from_millis(0));
            match res {
                Ok(NoiseEvent::Note(note)) => self.note = note,
                Ok(NoiseEvent::Envelope(e)) => self.envelope = e,
                Ok(NoiseEvent::KeyOff(k)) => self.keyoff = k,
                Err(_) => {}
            }

            let mut volume = self.envelope.next();
            if self.keyoff.next() {
                volume = 0.0;
            }
            info!("N ENV {:?}", self.envelope);
            info!("N KOF {:?}", self.keyoff);

            *x = if self.value { 0.0 } else { 1.0 } * volume;

            let last_phase = self.phase;
            self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
            if last_phase > self.phase {
                self.value = if self.note.is_long {
                    self.long_random.next()
                } else {
                    self.short_random.next()
                };
            }
        }
    }
}

struct NoiseRandom {
    bit: u8,
    value: u16,
}

impl NoiseRandom {
    pub fn long() -> Self {
        NoiseRandom { bit: 1, value: 1 }
    }

    pub fn short() -> Self {
        NoiseRandom { bit: 6, value: 1 }
    }

    pub fn next(&mut self) -> bool {
        // 15ビットシフトレジスタにはリセット時に1をセットしておく必要があります。 タイマによってシフトレジスタが励起されるたびに1ビット右シフトし、 ビット14には、ショートモード時にはビット0とビット6のEORを、 ロングモード時にはビット0とビット1のEORを入れます。
        // ロングモード時にはビット0とビット1のEORを入れます。
        let b = (self.value & 0x01) ^ ((self.value >> self.bit) & 0x01);
        self.value = self.value >> 1;
        self.value = self.value & 0b011_1111_1111_1111 | b << 14;

        // シフトレジスタのビット0が1なら、チャンネルの出力は0となります。
        self.value & 0x01 != 0
    }
}

fn init_noise(sdl_context: &sdl2::Sdl) -> (AudioDevice<NoiseWave>, Sender<NoiseEvent>) {
    let audio_subsystem = sdl_context.audio().unwrap();

    let (sender, receiver) = channel::<NoiseEvent>();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| NoiseWave {
            freq: spec.freq as f32,
            phase: 0.0,
            receiver: receiver,
            value: false,
            long_random: NoiseRandom::long(),
            short_random: NoiseRandom::short(),
            note: NoiseNote {
                hz: 0.0,
                is_long: true,
                volume: 0.0,
            },
            envelope: Envelope::new(0, false),
            keyoff: KeyOff::new(false, 0),
        })
        .unwrap();

    device.resume();

    (device, sender)
}
