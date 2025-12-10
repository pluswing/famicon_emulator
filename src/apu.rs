use crate::MAPPER;
use bitflags::bitflags;
use log::{debug, info, trace};
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use std::ops::Add;
use std::thread::sleep;
use std::time::{Duration, Instant};

const MASTER_VOLUME: f32 = 0.4;
const SAMPLE_RATE: f32 = 44100.0;

pub struct NesAPU {
    ch1_register: Ch1Register,
    ch2_register: Ch2Register,
    ch3_register: Ch3Register,
    ch4_register: Ch4Register,
    ch5_register: Ch5Register,

    frame_sequencer: FrameSequencer,
    status: StatusRegister,
    cycles: usize,
    counter: usize,

    device: AudioQueue<f32>,
    timer: Instant,
}

const NES_CPU_CLOCK: f32 = 1_789_772.5; // 1.78MHz

fn init_channel(sdl_context: &sdl2::Sdl) -> AudioQueue<f32> {
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(1),   // mono
        samples: Some(4410), // buffer size
    };

    let device = audio_subsystem.open_queue(None, &desired_spec).unwrap();
    device.resume();

    return device;
}

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let device = init_channel(sdl_context);

        NesAPU {
            ch1_register: Ch1Register::new(),
            ch2_register: Ch2Register::new(),
            ch3_register: Ch3Register::new(),
            ch4_register: Ch4Register::new(),
            ch5_register: Ch5Register::new(),
            frame_sequencer: FrameSequencer::new(),
            status: StatusRegister::new(),
            cycles: 0,
            counter: 0,

            device: device,
            timer: Instant::now(),
        }
    }

    pub fn write1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);
    }

    pub fn write2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);
    }

    pub fn write3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);
    }

    pub fn write4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);
    }

    pub fn write5ch(&mut self, addr: u16, value: u8) {
        self.ch5_register.write(addr, value);
    }

    pub fn read_status(&mut self) -> u8 {
        let mut res = self.status.bits();
        res = res & 0xF0;
        res = res
            | if self.ch1_register.length_counter == 0 {
                0
            } else {
                1
            };
        res = res
            | (if self.ch2_register.length_counter == 0 {
                0
            } else {
                1
            } << 1);

        res = res
            | (if self.ch3_register.length_counter == 0 {
                0
            } else {
                1
            } << 2);

        res = res
            | (if self.ch4_register.length_counter == 0 {
                0
            } else {
                1
            } << 3);

        res = res | (if self.ch5_register.is_active() { 1 } else { 0 } << 4);

        self.status.remove(StatusRegister::ENABLE_FRAME_IRQ);
        if self.status.contains(StatusRegister::ENABLE_DMC_IRQ) {
            self.status.remove(StatusRegister::ENABLE_DMC_IRQ);
            self.ch5_register.clear_irq();
        }
        res
    }

    pub fn write_status(&mut self, data: u8) {
        self.status.update(data);
        let dmc_enabled = self.status.contains(StatusRegister::ENABLE_5CH);
        self.ch5_register.set_enabled(dmc_enabled);
        if !dmc_enabled {
            self.status.remove(StatusRegister::ENABLE_DMC_IRQ);
            self.ch5_register.clear_irq();
        }
    }

    pub fn frame_irq(&self) -> bool {
        self.status.contains(StatusRegister::ENABLE_FRAME_IRQ)
    }

    pub fn write_frame_sequencer(&mut self, value: u8) {
        self.frame_sequencer.update(value);
        self.cycles = 0;
        self.counter = 0;
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let interval = 7457;
        if self.cycles > interval {
            self.cycles -= interval;
            self.counter += 1;

            // ここでクロック分の待ち時間を消費する
            let interval_ns = 1000 * 1000 * 1000 / 240;
            let time = self.timer.elapsed().as_nanos();
            if time < interval_ns {
                sleep(Duration::from_nanos((interval_ns - time) as u64));
                println!("sleep: {}", interval_ns - time);
            }
            let duration = self.timer.elapsed().as_nanos();
            self.timer = Instant::now();

            match self.frame_sequencer.mode() {
                4 => {
                    // - - - f      60 Hz
                    // - l - l     120 Hz
                    // e e e e     240 Hz
                    if self.counter == 2 || self.counter == 4 {
                        // 長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter == 4 {
                        // 割り込みフラグセット
                        self.counter = 0;
                        if self.frame_sequencer.irq() {
                            self.status.insert(StatusRegister::ENABLE_FRAME_IRQ);
                        }
                    }
                    // エンベロープと三角波の線形カウンタのクロック生成
                    self.send_envelope_tick();
                }
                5 => {
                    // - - - - -   (割り込みフラグはセットしない)
                    // - l - - l   96 Hz
                    // e e e - e  192 Hz

                    if self.counter == 1 || self.counter == 4 {
                        // 長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter != 3 {
                        // エンベロープと三角波の線形カウンタのクロック生成
                        self.send_envelope_tick();
                    }
                    if self.counter == 5 {
                        self.counter = 0;
                    }
                }
                _ => panic!("can't be"),
            }
            println!("duration: {}", duration);
            self.add_buffer();
        }
    }

    fn add_buffer(&mut self) {
        let must_add = SAMPLE_RATE / 240.0;
        let buffer_min_size = SAMPLE_RATE * (5.0 / 60.0);
        let buffer_size = self.device.size() as f32 / 4.0; // f32=4byteなので

        let add_buffer_size = if buffer_min_size > buffer_size {
            buffer_min_size - buffer_size
        } else {
            must_add
        };

        let mut buffer = vec![0.0; add_buffer_size as usize];
        for sample in buffer.iter_mut() {
            *sample = 0.0;
            if self.status.contains(StatusRegister::ENABLE_1CH) {
                *sample += self.ch1_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_2CH) {
                *sample += self.ch2_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_3CH) {
                *sample += self.ch3_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_4CH) {
                *sample += self.ch4_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_5CH) {
                *sample += self.ch5_register.next();
                if self.ch5_register.irq_pending() {
                    self.status.insert(StatusRegister::ENABLE_DMC_IRQ);
                }
            }
        }
        self.device.queue_audio(&buffer).unwrap();
    }

    fn send_envelope_tick(&mut self) {
        self.ch1_register.tick_envelope();
        self.ch2_register.tick_envelope();
        self.ch4_register.tick_envelope();

        self.ch3_register.tick_linear_counter();
    }

    fn send_length_counter_tick(&mut self) {
        self.ch1_register.tick_length_counter();
        self.ch2_register.tick_length_counter();
        self.ch3_register.tick_length_counter();
        self.ch4_register.tick_length_counter();
    }

    fn send_sweep_tick(&mut self) {
        self.ch1_register.tick_sweep();
        self.ch2_register.tick_sweep();
    }
}

// ########################################################################
// Registers
// ########################################################################

pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

static LENGTH_COUNTER_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

static NOISE_TABLE: [u16; 16] = [
    0x004, 0x008, 0x010, 0x020, 0x040, 0x060, 0x080, 0x0A0, 0x0CA, 0x0FE, 0x17C, 0x1FC, 0x2FA,
    0x3F8, 0x7F2, 0xFE4,
];

static FREQUENCY_TABLE: [u16; 16] = [
    0x1AC, 0x17C, 0x154, 0x140, 0x11E, 0x0FE, 0x0E2, 0x0D6, 0x0BE, 0x0A0, 0x08E, 0x080, 0x06A,
    0x054, 0x048, 0x036,
];

struct Ch1Register {
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    sweep_change_amount: u8,
    sweep_direction: u8,
    sweep_timer_count: u8,
    sweep_enabled: bool,

    frequency: u16,

    key_off_count: u8,

    // background status
    phase: f32,
    length_counter: u8,
    envelope_counter: u8,
    envelope_division_period: u8,
    sweep_counter: u8,
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
            sweep_enabled: false,

            frequency: 0,

            key_off_count: 0,

            phase: 0.0,
            length_counter: 0,
            envelope_counter: 0x0F,
            envelope_division_period: 1,
            sweep_counter: 0,
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
                self.sweep_enabled = (value & 0x80) != 0;
            }
            0x4002 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4003 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                // reset
                self.envelope_reset();
                self.sweep_counter = 0;
                self.phase = 0.0;
            }
            _ => panic!("can't be"),
        }
    }

    fn next(&mut self) -> f32 {
        if self.mute() {
            return 0.0;
        }

        let duty = self.duty();
        let step = ((self.phase * 8.0).floor() as usize) % 8;
        let x = if duty[step] == 0 {
            -self.volume()
        } else {
            self.volume()
        } * MASTER_VOLUME;

        let hz = self.hz();
        if hz != 0.0 {
            self.phase = (self.phase + hz / SAMPLE_RATE) % 1.0;
        }
        return x;
    }

    fn duty(&self) -> &'static [u8; 8] {
        &DUTY_TABLE[self.duty as usize]
    }

    fn tick_envelope(&mut self) {
        self.envelope_division_period -= 1;
        if self.envelope_division_period != 0 {
            return;
        }

        // 分周器が励起 => division_period==0
        if self.envelope_counter != 0 {
            self.envelope_counter -= 1;
        } else if self.envelope_counter == 0 {
            if self.key_off_counter_flag {
                self.envelope_reset();
            }
        }
        self.envelope_division_period = self.volume + 1;
    }

    fn volume(&self) -> f32 {
        (if self.envelope_flag {
            self.envelope_counter
        } else {
            self.volume
        }) as f32
            / 15.0
    }

    fn envelope_reset(&mut self) {
        self.envelope_counter = 0x0F;
        self.envelope_division_period = self.volume + 1;
    }

    fn tick_length_counter(&mut self) {
        if !self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn mute(&self) -> bool {
        self.length_counter == 0
    }

    fn tick_sweep(&mut self) {
        // チャンネルの長さカウンタがゼロではない
        if self.mute() {
            return;
        }
        self.sweep_counter += 1;
        if self.sweep_counter < (self.sweep_timer_count + 1) {
            return;
        }
        self.sweep_counter = 0;

        if !self.key_off_counter_flag {
            return;
        }
        if self.sweep_change_amount == 0 {
            return;
        }
        if self.sweep_direction == 0 {
            // しり下がりモード    新しい周期 = 周期 + (周期 >> N)
            self.frequency = self.frequency + (self.frequency >> self.sweep_change_amount);
        } else {
            // しり上がりモード    新しい周期 = 周期 - (周期 >> N)
            self.frequency = self.frequency - (self.frequency >> self.sweep_change_amount);
        }

        // もしチャンネルの周期が8未満か、$7FFより大きくなったなら、スイープを停止し、 チャンネルを無音化します。
        if self.frequency < 0x08 || self.frequency > 0x7FF {
            self.key_off_count = 0;
            self.length_counter = 0;
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0;
        }
        NES_CPU_CLOCK / (16.0 * (self.frequency as f32 + 1.0))
    }

    fn reset(&mut self) {
        self.sweep_counter = 0;
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
    sweep_enabled: bool,

    frequency: u16,

    key_off_count: u8,

    // background status
    phase: f32,
    length_counter: u8,
    envelope_counter: u8,
    envelope_division_period: u8,
    sweep_counter: u8,
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
            sweep_enabled: false,

            frequency: 0,

            key_off_count: 0,

            phase: 0.0,
            length_counter: 0,
            envelope_counter: 0x0F,
            envelope_division_period: 1,
            sweep_counter: 0,
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
                self.sweep_enabled = (value & 0x80) != 0;
            }
            0x4006 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4007 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                // reset
                self.envelope_reset();
                self.sweep_counter = 0;
                self.phase = 0.0;
            }
            _ => panic!("can't be"),
        }
    }

    fn next(&mut self) -> f32 {
        if self.mute() {
            return 0.0;
        }

        let duty = self.duty();
        let step = ((self.phase * 8.0).floor() as usize) % 8;
        let x = if duty[step] == 0 {
            -self.volume()
        } else {
            self.volume()
        } * MASTER_VOLUME;

        let hz = self.hz();
        if hz != 0.0 {
            self.phase = (self.phase + hz / SAMPLE_RATE) % 1.0;
        }
        return x;
    }

    fn duty(&self) -> &'static [u8; 8] {
        &DUTY_TABLE[self.duty as usize]
    }

    fn tick_envelope(&mut self) {
        self.envelope_division_period -= 1;
        if self.envelope_division_period != 0 {
            return;
        }

        // 分周器が励起 => division_period==0
        if self.envelope_counter != 0 {
            self.envelope_counter -= 1;
        } else if self.envelope_counter == 0 {
            if self.key_off_counter_flag {
                self.envelope_reset();
            }
        }
        self.envelope_division_period = self.volume + 1;
    }

    fn volume(&self) -> f32 {
        (if self.envelope_flag {
            self.envelope_counter
        } else {
            self.volume
        }) as f32
            / 15.0
    }

    fn envelope_reset(&mut self) {
        self.envelope_counter = 0x0F;
        self.envelope_division_period = self.volume + 1;
    }

    fn tick_length_counter(&mut self) {
        if !self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn mute(&self) -> bool {
        self.length_counter == 0
    }

    fn tick_sweep(&mut self) {
        self.sweep_counter += 1;
        if self.sweep_counter < (self.sweep_timer_count + 1) {
            return;
        }
        self.sweep_counter = 0;

        if !self.key_off_counter_flag {
            return;
        }
        if self.sweep_change_amount == 0 {
            return;
        }
        // チャンネルの長さカウンタがゼロではない
        if self.mute() {
            return;
        }

        if self.sweep_direction == 0 {
            // しり下がりモード    新しい周期 = 周期 + (周期 >> N)
            self.frequency = self.frequency + (self.frequency >> self.sweep_change_amount);
        } else {
            // しり上がりモード    新しい周期 = 周期 - (周期 >> N)
            self.frequency = self.frequency - (self.frequency >> self.sweep_change_amount);
        }

        // もしチャンネルの周期が8未満か、$7FFより大きくなったなら、スイープを停止し、 チャンネルを無音化します。
        if self.frequency < 0x08 || self.frequency > 0x7FF {
            self.key_off_count = 0;
            self.length_counter = 0;
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0;
        }
        NES_CPU_CLOCK / (16.0 * (self.frequency as f32 + 1.0))
    }

    fn reset(&mut self) {
        self.sweep_counter = 0;
    }
}

struct Ch3Register {
    // 4008
    length: u8,
    key_off_counter_flag: bool,

    // 400A, 400B
    frequency: u16,
    key_off_count: u8,

    phase: f32,
    linear_counter: u8,
    length_counter: u8,
}

impl Ch3Register {
    pub fn new() -> Self {
        Ch3Register {
            length: 0,
            key_off_counter_flag: false,
            frequency: 0,
            key_off_count: 0,

            phase: 0.0,
            linear_counter: 0,
            length_counter: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4008 => {
                self.length = value & 0x7F;
                self.linear_counter = self.length;
                self.key_off_counter_flag = (value & 0x80) == 0;
            }
            0x4009 => {}
            0x400A => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x400B => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];
                self.linear_counter = self.length;

                self.phase = 0.0;
            }
            _ => panic!("can't be"),
        }
    }

    fn next(&mut self) -> f32 {
        let mut x = (if self.phase <= 0.5 {
            self.phase
        } else {
            1.0 - self.phase
        } - 0.25)
            * 4.0
            * MASTER_VOLUME;

        if self.length_counter == 0 {
            x = 0.0;
        }
        if self.linear_counter == 0 {
            x = 0.0;
        }
        self.phase = (self.phase + self.hz() / SAMPLE_RATE) % 1.0;
        return x;
    }

    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / (32.0 * (self.frequency as f32 + 1.0))
    }

    fn tick_length_counter(&mut self) {
        if !self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn tick_linear_counter(&mut self) {
        if !self.key_off_counter_flag {
            return;
        }
        if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NoiseKind {
    Long,
    Short,
}

struct Ch4Register {
    // 400C
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,

    // 400E
    frequency: u8,
    kind: NoiseKind,

    // 400F
    key_off_count: u8,

    // background status
    phase: f32,
    random: u16,
    length_counter: u8,
    envelope_counter: u8,
    envelope_division_period: u8,
}

impl Ch4Register {
    pub fn new() -> Self {
        Ch4Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            frequency: 0,
            kind: NoiseKind::Long,
            key_off_count: 0,
            phase: 0.0,
            random: 1, // 重要
            length_counter: 0,
            envelope_counter: 0x0F,
            envelope_division_period: 1,
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
                self.kind = match value & 0x80 {
                    0 => NoiseKind::Long,
                    _ => NoiseKind::Short,
                };
            }
            0x400F => {
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                self.phase = 0.0;
                self.random = 1;
                self.envelope_reset();
            }
            _ => panic!("can't be"),
        }
    }

    fn next(&mut self) -> f32 {
        let mut x =
            if (self.random & 0x01) != 0 { 0.0 } else { 1.0 } * self.volume() * MASTER_VOLUME;

        if self.length_counter == 0 {
            x = 0.0;
        }

        let last_phase = self.phase;
        let mut add = self.hz() / SAMPLE_RATE;
        self.phase = (self.phase + add) % 1.0;

        loop {
            if add < 1.0 {
                break;
            }
            add -= 1.0;
            self.next_random();
        }

        if self.phase < last_phase {
            self.next_random();
        }
        return x;
    }

    fn volume(&self) -> f32 {
        (if self.envelope_flag {
            self.envelope_counter
        } else {
            self.volume
        }) as f32
            / 15.0
    }

    pub fn next_random(&mut self) {
        let bit1 = (self.random >> (if self.kind == NoiseKind::Long { 1 } else { 6 })) & 0x01;
        let bit2 = self.random & 0x01;
        self.random = (self.random >> 1) | (bit1 ^ bit2) << 14;
    }

    fn tick_length_counter(&mut self) {
        if !self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    fn tick_envelope(&mut self) {
        self.envelope_division_period -= 1;
        if self.envelope_division_period != 0 {
            return;
        }

        // 分周器が励起 => division_period==0
        if self.envelope_counter != 0 {
            self.envelope_counter -= 1;
        } else if self.envelope_counter == 0 {
            if self.key_off_counter_flag {
                self.envelope_reset();
            }
        }
        self.envelope_division_period = self.volume + 1;
    }

    fn envelope_reset(&mut self) {
        self.envelope_counter = 0x0F;
        self.envelope_division_period = self.volume + 1;
    }

    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / NOISE_TABLE[self.frequency as usize] as f32
    }
}

struct Ch5Register {
    // 4010-4013 registers
    irq_enabled: bool,
    loop_flag: bool,
    frequency_index: u8,
    delta_counter: u8,
    start_addr: u8,
    byte_count: u8,

    // runtime state
    enabled: bool,
    phase: f32,
    data: u8,
    sample_addr: u16,
    counter: u32,
    irq_pending: bool,
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
            enabled: false,
            phase: 0.0,
            data: 0,
            sample_addr: 0xC000,
            counter: Self::counter_from_length(0),
            irq_pending: false,
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
                self.byte_count = 1;
                self.counter = Self::counter_from_length(1);
            }
            0x4012 => {
                self.start_addr = value;
                self.sample_addr = value as u16 * 0x40 + 0xC000;
            }
            0x4013 => {
                self.byte_count = value;
                self.counter = Self::counter_from_length(value);
            }
            _ => panic!("can't be"),
        }
    }

    fn frequency(&self) -> f32 {
        NES_CPU_CLOCK / FREQUENCY_TABLE[self.frequency_index as usize] as f32
    }

    pub fn next(&mut self) -> f32 {
        if !self.enabled {
            return 0.0;
        }

        let last_phase = self.phase;
        self.phase = (self.phase + self.frequency() / SAMPLE_RATE) % 1.0;

        if last_phase > self.phase {
            self.clock();
        }

        if self.delta_counter == 0 || self.counter == 0 {
            0.0
        } else {
            ((self.delta_counter as f32 - 64.0) / 64.0) * MASTER_VOLUME
        }
    }

    fn clock(&mut self) {
        if self.counter == 0 {
            return;
        }

        if self.counter & 0x0007 == 0 {
            self.fetch_sample_byte();
        }

        self.update_delta();
        self.counter -= 1;

        if self.counter == 0 {
            if self.loop_flag {
                self.set_delta();
            } else if self.irq_enabled {
                self.irq_pending = true;
            }
        }
    }

    fn fetch_sample_byte(&mut self) {
        if self.counter == 0 {
            return;
        }

        unsafe {
            self.data = MAPPER.read_prg_rom(self.sample_addr);
        }
        if self.sample_addr == 0xFFFF {
            self.sample_addr = 0x8000;
        } else {
            self.sample_addr = self.sample_addr.wrapping_add(1);
        }
    }

    fn update_delta(&mut self) {
        if self.data & 0x01 == 0x00 {
            if self.delta_counter > 1 {
                self.delta_counter -= 2
            }
        } else if self.delta_counter < 126 {
            self.delta_counter += 2
        }
        self.data >>= 1;
    }

    fn set_delta(&mut self) {
        self.sample_addr = self.start_addr as u16 * 0x40 + 0xC000;
        self.counter = Self::counter_from_length(self.byte_count);
        self.data = 0;
        self.irq_pending = false;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.counter = 0;
            self.irq_pending = false;
        } else if self.counter == 0 && self.byte_count != 0 {
            self.set_delta();
        }
    }

    pub fn is_active(&self) -> bool {
        self.enabled && self.counter > 0
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    pub fn clear_irq(&mut self) {
        self.irq_pending = false;
    }

    fn counter_from_length(len: u8) -> u32 {
        (len as u32 * 8) * 0x10 + 1
    }
}

bitflags! {
  pub struct FrameSequencer: u8 {
    const DISABLE_IRQ    = 0b0100_0000;
    const SEQUENCER_MODE = 0b1000_0000;
  }
}

impl FrameSequencer {
    pub fn new() -> Self {
        FrameSequencer::empty()
    }

    pub fn mode(&self) -> u8 {
        if self.contains(FrameSequencer::SEQUENCER_MODE) {
            5
        } else {
            4
        }
    }

    pub fn irq(&self) -> bool {
        !self.contains(FrameSequencer::DISABLE_IRQ)
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}

bitflags! {
  pub struct StatusRegister: u8 {
    const ENABLE_1CH       = 0b0000_0001;
    const ENABLE_2CH       = 0b0000_0010;
    const ENABLE_3CH       = 0b0000_0100;
    const ENABLE_4CH       = 0b0000_1000;
    const ENABLE_5CH       = 0b0001_0000;

    const ENABLE_FRAME_IRQ = 0b0100_0000;
    const ENABLE_DMC_IRQ   = 0b1000_0000;
  }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::empty()
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}
