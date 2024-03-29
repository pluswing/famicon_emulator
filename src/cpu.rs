use log::{debug, info, trace};

use crate::opscodes::{call, CPU_OPS_CODES};
use crate::MAPPER;

use crate::bus::{Bus, Mem};

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    Relative,
    Implied,
    NoneAddressing,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CycleCalcMode {
    None,
    Page,
    Branch,
}

#[derive(Debug, Clone)]
pub struct OpCode {
    pub code: u8,
    pub name: String,
    pub bytes: u16,
    pub cycles: u8,
    pub cycle_calc_mode: CycleCalcMode,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(
        code: u8,
        name: &str,
        bytes: u16,
        cycles: u8,
        cycle_calc_mode: CycleCalcMode,
        addressing_mode: AddressingMode,
    ) -> Self {
        OpCode {
            code: code,
            name: String::from(name),
            bytes: bytes,
            cycles: cycles,
            cycle_calc_mode: cycle_calc_mode,
            addressing_mode: addressing_mode,
        }
    }
}

const FLAG_CARRY: u8 = 1 << 0;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_INTERRRUPT: u8 = 1 << 2;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_BREAK: u8 = 1 << 4;
const FLAG_BREAK2: u8 = 1 << 5; // 5 は未使用。
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_NEGATIVE: u8 = 1 << 7;

const SIGN_BIT: u8 = 1 << 7;

pub struct CPU<'a> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    // pub memory: [u8; 0x10000], // 0xFFFF
    pub bus: Bus<'a>,

    add_cycles: u8,
}

pub static mut IN_TRACE: bool = false;

impl Mem for CPU<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data)
    }
}

impl<'a> CPU<'a> {
    pub fn new(bus: Bus<'a>) -> CPU<'a> {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: FLAG_INTERRRUPT | FLAG_BREAK2, // FIXME あってる？
            program_counter: 0,
            stack_pointer: 0xFD, // FIXME あってる？
            // memory: [0x00; 0x10000],
            bus: bus,
            add_cycles: 0,
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Implied => {
                panic!("AddressingMode::Implied");
            }
            AddressingMode::Accumulator => {
                panic!("AddressingMode::Accumulator");
            }
            // LDA #$44 => a9 44
            AddressingMode::Immediate => self.program_counter,

            // LDA $44 => a5 44
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            // LDA $4400 => ad 00 44
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            // LDA $44,X => b5 44
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }

            // LDX $44,Y => b6 44
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            // LDA $4400,X => bd 00 44
            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_x as u16);
                // (+1 if page crossed)
                if base & 0xFF00 != addr & 0xFF00 {
                    self.add_cycles += 1;
                }
                addr
            }

            // LDA $4400,Y => b9 00 44
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
                // (+1 if page crossed)
                if base & 0xFF00 != addr & 0xFF00 {
                    self.add_cycles += 1;
                }
                addr
            }
            // JMP -> same Absolute
            AddressingMode::Indirect => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = self.mem_read_u16(base);
                addr
            }

            // LDA ($44,X) => a1 44
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let addr = self.mem_read_u16(ptr as u16);
                addr
            }

            // LDA ($44),Y => b1 44
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);
                let deref_base = self.mem_read_u16(base as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                // (+1 if page crossed)
                if deref_base & 0xFF00 != deref & 0xFF00 {
                    self.add_cycles += 1;
                }
                deref
            }

            // BCC *+4 => 90 04
            AddressingMode::Relative => {
                let base = self.mem_read(self.program_counter);
                let np = (base as i8) as i32 + self.program_counter as i32;
                return np as u16;
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn mem_read_u16(&mut self, pos: u16) -> u16 {
        // FIXME
        if pos == 0x00FF || pos == 0x02FF {
            debug!("mem_read_u16 page boundary. {:04X}", pos);
            let lo = self.mem_read(pos) as u16;
            let hi = self.mem_read(pos & 0xFF00) as u16;
            return (hi << 8) | (lo as u16);
        }
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    fn load_and_run(&mut self, program: Vec<u8>) {
        self.load();
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        // FIXME あってる？
        self.status = FLAG_INTERRRUPT | FLAG_BREAK2;
        self.stack_pointer = 0xFD;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self) {
        // self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status() {
                self.interrupt_nmi();
            }

            if self.bus.poll_apu_irq() {
                self.apu_irq();
            } else if unsafe { MAPPER.is_irq() } {
                self.apu_irq();
            }

            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let op = CPU_OPS_CODES.get(&opscode);
            match op {
                Some(op) => {
                    self.add_cycles = 0;

                    callback(self);
                    call(self, &op);

                    match op.cycle_calc_mode {
                        CycleCalcMode::None => {
                            self.add_cycles = 0;
                        }
                        CycleCalcMode::Page => {
                            if self.add_cycles > 1 {
                                panic!(
                                    "Unexpected cycle_calc. {} {:?} => {}",
                                    op.name, op.addressing_mode, self.add_cycles
                                )
                            }
                        }
                        _ => {}
                    }

                    self.bus.tick(op.cycles + self.add_cycles);

                    // if program_conter_state == self.program_counter {
                    //   self.program_counter += (op.len - 1) as u16
                    // }
                }
                _ => {} // panic!("no implementation {:<02X}", opscode),
            }
        }
    }

    fn interrupt_nmi(&mut self) {
        debug!("** INTERRUPT_NMI **");
        self._push_u16(self.program_counter);
        let mut status = self.status;
        status = status & !FLAG_BREAK;
        status = status | FLAG_BREAK2;
        self._push(status);

        self.status = self.status | FLAG_INTERRRUPT;
        self.bus.tick(2);
        self.program_counter = self.mem_read_u16(0xFFFA);
    }

    fn apu_irq(&mut self) {
        if self.status & FLAG_INTERRRUPT != 0 {
            return;
        }
        self._push_u16(self.program_counter);
        self._push(self.status);
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    pub fn anc(&mut self, mode: &AddressingMode) {
        todo!("anc")
    }
    pub fn arr(&mut self, mode: &AddressingMode) {
        todo!("arr")
    }
    pub fn asr(&mut self, mode: &AddressingMode) {
        todo!("asr")
    }
    pub fn lxa(&mut self, mode: &AddressingMode) {
        todo!("lxa")
    }
    pub fn sha(&mut self, mode: &AddressingMode) {
        todo!("sha")
    }
    pub fn sbx(&mut self, mode: &AddressingMode) {
        //  A&X minus #{imm} into X
        // AND X register with accumulator and store result in X regis-ter, then
        // subtract byte from X register (without borrow).
        // Status flags: N,Z,C

        // AND X をアキュムレータに登録し、結果を X レジスタに格納します。 X レジスタからバイトを減算します (ボローなし)。 ステータスフラグ：N、Z、C
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let (v, overflow) = (self.register_a & self.register_x).overflowing_sub(value);
        self.register_x = v;
        self.update_zero_and_negative_flags(self.register_x);
        self.status = if overflow {
            self.status & FLAG_OVERFLOW
        } else {
            self.status | FLAG_OVERFLOW
        };
        todo!("sbx")
    }

    pub fn jam(&mut self, mode: &AddressingMode) {
        // Stop program counter (processor lock up).
        self.program_counter -= 1;
        panic!("CALL JAM operation.");
    }

    pub fn lae(&mut self, mode: &AddressingMode) {
        // stores {adr}&S into A, X and S

        // AND memory with stack pointer, transfer result to accu-mulator, X
        // register and stack pointer.
        // Status flags: N,Z
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        let s = self._pop();
        self.register_a = value & s;
        self.register_x = self.register_a;
        self._push(self.register_a);
        self.update_zero_and_negative_flags(self.register_a);
        todo!("lae")
    }

    pub fn shx(&mut self, mode: &AddressingMode) {
        // M =3D X AND HIGH(arg) + 1
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_x & h).wrapping_add(1));
        todo!("shx")
    }

    pub fn shy(&mut self, mode: &AddressingMode) {
        // Y&H into {adr}
        // AND Y register with the high byte of the target address of the argument
        // + 1. Store the result in memory.
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_y & h).wrapping_add(1));
        todo!("shy")
    }

    pub fn ane(&mut self, mode: &AddressingMode) {
        // TXA + AND #{imm}
        self.txa(mode);
        self.and(mode);
        todo!("ane")
    }

    pub fn shs(&mut self, mode: &AddressingMode) {
        // stores A&X into S and A&X&H into {adr}
        // アキュムレータと X レジスタを AND 演算し、結果をスタック ポインタに格納します。次に、スタック ポインタと引数 1 のターゲット アドレスの上位バイトを AND 演算します。結果をメモリに格納します。
        self._push(self.register_a & self.register_x);
        let addr = self.get_operand_address(mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, self.register_a & self.register_x & h);
        todo!("shs")
    }

    pub fn rra(&mut self, mode: &AddressingMode) {
        self.ror(mode);
        self.adc(mode);
    }

    pub fn sre(&mut self, mode: &AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    pub fn rla(&mut self, mode: &AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    pub fn slo(&mut self, mode: &AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    pub fn isb(&mut self, mode: &AddressingMode) {
        // = ISC
        self.inc(mode);
        self.sbc(mode);
    }

    pub fn dcp(&mut self, mode: &AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
    }

    pub fn sax(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a & self.register_x);
    }

    pub fn lax(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax(mode);
    }

    pub fn txs(&mut self, mode: &AddressingMode) {
        self.stack_pointer = self.register_x;
    }

    pub fn tsx(&mut self, mode: &AddressingMode) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn tya(&mut self, mode: &AddressingMode) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tay(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn txa(&mut self, mode: &AddressingMode) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tax(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    pub fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    pub fn rti(&mut self, mode: &AddressingMode) {
        // スタックからプロセッサ フラグをプルし、続いてプログラム カウンタをプルします。
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
        self.program_counter = self._pop_u16();
    }

    pub fn plp(&mut self, mode: &AddressingMode) {
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
    }

    pub fn php(&mut self, mode: &AddressingMode) {
        self._push(self.status | FLAG_BREAK | FLAG_BREAK2);
    }

    pub fn pla(&mut self, mode: &AddressingMode) {
        self.register_a = self._pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn pha(&mut self, mode: &AddressingMode) {
        self._push(self.register_a);
    }

    pub fn nop(&mut self, mode: &AddressingMode) {
        // なにもしない
    }

    pub fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn rts(&mut self, mode: &AddressingMode) {
        let value = self._pop_u16() + 1;
        self.program_counter = value;
    }

    pub fn jsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self._push_u16(self.program_counter + 2 - 1);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter -= 2;
    }

    pub fn _push(&mut self, value: u8) {
        let addr = 0x0100 + self.stack_pointer as u16;
        trace!("STACK PUSH: {:04X} => {:02X}", self.stack_pointer, value);
        self.mem_write(addr, value);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn _pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        let addr = 0x0100 + self.stack_pointer as u16;
        trace!("STACK POP: {:02X}", self.stack_pointer);
        self.mem_read(addr)
    }

    pub fn _push_u16(&mut self, value: u16) {
        self._push((value >> 8) as u8);
        self._push((value & 0x00FF) as u8);
    }

    pub fn _pop_u16(&mut self) -> u16 {
        let lo = self._pop();
        let hi = self._pop();
        ((hi as u16) << 8) | lo as u16
    }

    pub fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter -= 2;
        // TODO
        // オリジナルの 6502 は、間接ベクトルがページ境界にある場合、ターゲット アドレスを正しくフェッチしません (たとえば、$xxFF で、xx は $00 から $FF までの任意の値です)。この場合、予想どおり $xxFF から LSB を取得しますが、$xx00 から MSB を取得します。これは、65SC02 などの最近のチップで修正されているため、互換性のために、間接ベクトルがページの最後にないことを常に確認してください。
    }

    pub fn iny(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn inx(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn dey(&mut self, mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn dex(&mut self, mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr).wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn _cmp(&mut self, target: u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        if target >= value {
            self.sec(&AddressingMode::Implied);
        } else {
            self.clc(&AddressingMode::Implied);
        }
        let value = target.wrapping_sub(value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn cpy(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_y, mode);
    }

    pub fn cpx(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_x, mode);
    }

    pub fn cmp(&mut self, mode: &AddressingMode) {
        self._cmp(self.register_a, mode);
    }

    pub fn clv(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_OVERFLOW;
    }

    pub fn sei(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_INTERRRUPT;
    }

    pub fn cli(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_INTERRRUPT;
    }

    pub fn sed(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_DECIMAL;
    }

    pub fn cld(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_DECIMAL;
    }

    pub fn sec(&mut self, mode: &AddressingMode) {
        self.status = self.status | FLAG_CARRY;
    }

    pub fn clc(&mut self, mode: &AddressingMode) {
        self.status = self.status & !FLAG_CARRY;
    }

    pub fn bvs(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_OVERFLOW, true);
    }

    pub fn bvc(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_OVERFLOW, false);
    }

    fn _branch(&mut self, mode: &AddressingMode, flag: u8, nonzero: bool) {
        let addr = self.get_operand_address(mode);
        if nonzero {
            if self.status & flag != 0 {
                // (+1 if branch succeeds
                //  +2 if to a new page)
                //    => new pageの場合は、+1っぽい。
                //     https://pgate1.at-ninja.jp/NES_on_FPGA/nes_cpu.htm#clock
                self.add_cycles += 1;
                if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                    self.add_cycles += 1;
                }
                self.program_counter = addr
            }
        } else {
            if self.status & flag == 0 {
                // (+1 if branch succeeds
                //  +2 if to a new page)
                self.add_cycles += 1;
                if (self.program_counter & 0xFF00) != (addr & 0xFF00) {
                    self.add_cycles += 1;
                }
                self.program_counter = addr
            }
        }
    }

    pub fn brk(&mut self, mode: &AddressingMode) {
        // FLAG_BREAK が立っている場合は
        if self.status & FLAG_BREAK != 0 {
            return;
        }

        // プログラム カウンターとプロセッサ ステータスがスタックにプッシュされ、
        self._push_u16(self.program_counter + 1);
        self._push(self.status);

        // $FFFE/F の IRQ 割り込みベクトルが PC にロードされ、ステータスのブレーク フラグが 1 に設定されます。
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    pub fn bpl(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_NEGATIVE, false);
    }

    pub fn bmi(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_NEGATIVE, true);
    }

    pub fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let zero = self.register_a & value;
        if zero == 0 {
            self.status = self.status | FLAG_ZERO;
        } else {
            self.status = self.status & !FLAG_ZERO;
        }
        let flags = FLAG_NEGATIVE | FLAG_OVERFLOW;
        self.status = (self.status & !flags) | (value & flags);
    }

    pub fn bne(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_ZERO, false);
    }

    pub fn beq(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_ZERO, true);
    }

    pub fn bcc(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_CARRY, false);
    }

    pub fn bcs(&mut self, mode: &AddressingMode) {
        self._branch(mode, FLAG_CARRY, true);
    }

    pub fn ror(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            self.register_a = self.register_a | ((self.status & FLAG_CARRY) << 7);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            let value = value | ((self.status & FLAG_CARRY) << 7);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn rol(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value | (self.status & FLAG_CARRY);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            let value = value | (self.status & FLAG_CARRY);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn lsr(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let carry = value & 0x01;
            let value = value / 2;
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry == 1 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn asl(&mut self, mode: &AddressingMode) {
        let (value, carry) = if mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value;
            (value, carry)
        } else {
            let addr = self.get_operand_address(mode);
            let value = self.mem_read(addr);
            let (value, carry) = value.overflowing_mul(2);
            self.mem_write(addr, value);
            (value, carry)
        };

        self.status = if carry {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.update_zero_and_negative_flags(value);
    }

    pub fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn sbc(&mut self, mode: &AddressingMode) {
        // A-M-(1-C)
        // キャリーかどうかの判定が逆
        // キャリーの引き算(1-C)
        // overflowの判定が逆 = m,p, p,m
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (v1, carry_flag1) = self.register_a.overflowing_sub(value);
        let (n, carry_flag2) = v1.overflowing_sub(1 - carry);

        let overflow = (self.register_a & SIGN_BIT) != (value & SIGN_BIT)
            && (self.register_a & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if !carry_flag1 && !carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a)
    }

    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let carry = self.status & FLAG_CARRY;
        let (rhs, carry_flag1) = value.overflowing_add(carry);
        let (n, carry_flag2) = self.register_a.overflowing_add(rhs);

        let overflow = (self.register_a & SIGN_BIT) == (value & SIGN_BIT)
            && (value & SIGN_BIT) != (n & SIGN_BIT);

        self.register_a = n;

        self.status = if carry_flag1 || carry_flag2 {
            self.status | FLAG_CARRY
        } else {
            self.status & !FLAG_CARRY
        };
        self.status = if overflow {
            self.status | FLAG_OVERFLOW
        } else {
            self.status & !FLAG_OVERFLOW
        };

        self.update_zero_and_negative_flags(self.register_a)
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status = if result == 0 {
            self.status | FLAG_ZERO
        } else {
            self.status & !FLAG_ZERO
        };

        self.status = if result & 0x80 != 0 {
            self.status | FLAG_NEGATIVE
        } else {
            self.status & !FLAG_NEGATIVE
        }
    }
}

pub fn trace(cpu: &mut CPU) -> String {
    // 0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD
    // OK 0064 => program_counter
    // OK A2 01 => binary code
    // OK LDX #$01 => asm code
    // "0400 @ 0400 = AA" => memory access
    // OK A:01 X:02 Y:03 P:24 SP:FD => register, status, stack_pointer
    unsafe { IN_TRACE = true };

    let program_counter = cpu.program_counter - 1;
    let pc = format!("{:<04X}", program_counter);
    let op = cpu.mem_read(program_counter);
    let ops = CPU_OPS_CODES.get(&op).unwrap();
    let mut args: Vec<u8> = vec![];
    for n in 1..ops.bytes {
        let arg = cpu.mem_read(program_counter + n);
        args.push(arg);
    }
    let bin = binary(op, &args);
    let asm = disasm(program_counter, &ops, &args);
    let memacc = memory_access(cpu, &ops, &args);
    let status = cpu2str(cpu);

    let log = format!(
        "{:<6}{:<9}{:<33}{}",
        pc,
        bin,
        vec![asm, memacc].join(" "),
        status
    );

    trace!("{}", log);

    unsafe { IN_TRACE = false };

    log
}

fn binary(op: u8, args: &Vec<u8>) -> String {
    let mut list: Vec<String> = vec![];
    list.push(format!("{:<02X}", op));
    for v in args {
        list.push(format!("{:<02X}", v));
    }
    list.join(" ")
}

fn disasm(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
    let prefix = if ops.name.starts_with("*") { "" } else { " " };
    format!(
        "{}{} {}",
        prefix,
        ops.name,
        address(program_counter, &ops, args)
    )
}

fn address(program_counter: u16, ops: &OpCode, args: &Vec<u8>) -> String {
    match ops.addressing_mode {
        AddressingMode::Implied => {
            format!("")
        }
        AddressingMode::Accumulator => {
            format!("A")
        }
        // LDA #$44 => a9 44
        AddressingMode::Immediate => {
            format!("#${:<02X}", args[0])
        }

        // LDA $44 => a5 44
        AddressingMode::ZeroPage => {
            format!("${:<02X}", args[0])
        }

        // LDA $4400 => ad 00 44
        AddressingMode::Absolute => {
            format!("${:<02X}{:<02X}", args[1], args[0])
        }
        // LDA $44,X => b5 44
        AddressingMode::ZeroPage_X => {
            format!("${:<02X},X", args[0])
        }

        // LDX $44,Y => b6 44
        AddressingMode::ZeroPage_Y => {
            format!("${:<02X},Y", args[0])
        }

        // LDA $4400,X => bd 00 44
        AddressingMode::Absolute_X => {
            format!("${:<02X}{:<02X},X", args[1], args[0])
        }

        // LDA $4400,Y => b9 00 44
        AddressingMode::Absolute_Y => {
            format!("${:<02X}{:<02X},Y", args[1], args[0])
        }
        // JMP
        AddressingMode::Indirect => {
            format!("(${:<02X}{:<02X})", args[1], args[0])
        }

        // LDA ($44,X) => a1 44
        AddressingMode::Indirect_X => {
            format!("(${:<02X},X)", args[0])
        }

        // LDA ($44),Y => b1 44
        AddressingMode::Indirect_Y => {
            format!("(${:<02X}),Y", args[0])
        }

        // BCC *+4 => 90 04
        AddressingMode::Relative => {
            format!(
                "${:<04X}",
                (program_counter as i32 + (args[0] as i8) as i32) as u16 + 2
            )
        }

        AddressingMode::NoneAddressing => {
            panic!("mode {:?} is not supported", ops.addressing_mode);
        }
    }
}

fn memory_access(cpu: &mut CPU, ops: &OpCode, args: &Vec<u8>) -> String {
    if ops.name.starts_with("J") {
        if ops.addressing_mode == AddressingMode::Indirect {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let addr = hi << 8 | lo;
            let value = cpu.mem_read_u16(addr);
            return format!("= {:<04X}", value);
        }
        return format!("");
    }

    match ops.addressing_mode {
        AddressingMode::ZeroPage => {
            let value = cpu.mem_read(args[0] as u16);
            format!("= {:<02X}", value)
        }
        AddressingMode::ZeroPage_X => {
            let addr = args[0].wrapping_add(cpu.register_x) as u16;
            let value = cpu.mem_read(addr);
            format!("@ {:<02X} = {:<02X}", addr, value)
        }
        AddressingMode::ZeroPage_Y => {
            let addr = args[0].wrapping_add(cpu.register_y) as u16;
            let value = cpu.mem_read(addr);
            format!("@ {:<02X} = {:<02X}", addr, value)
        }
        AddressingMode::Absolute => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let addr = hi << 8 | lo;
            let value = cpu.mem_read(addr);
            format!("= {:<02X}", value)
        }
        AddressingMode::Absolute_X => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let base = hi << 8 | lo;
            let addr = base.wrapping_add(cpu.register_x as u16);
            let value = cpu.mem_read(addr);
            format!("@ {:<04X} = {:<02X}", addr, value)
        }
        AddressingMode::Absolute_Y => {
            let hi = args[1] as u16;
            let lo = args[0] as u16;
            let base = hi << 8 | lo;
            let addr = base.wrapping_add(cpu.register_y as u16);
            let value = cpu.mem_read(addr);
            format!("@ {:<04X} = {:<02X}", addr, value)
        }
        AddressingMode::Indirect_X => {
            let base = args[0];
            let ptr: u8 = (base as u8).wrapping_add(cpu.register_x);
            let addr = cpu.mem_read_u16(ptr as u16);
            let value = cpu.mem_read(addr);
            format!("@ {:<02X} = {:<04X} = {:<02X}", ptr, addr, value)
        }
        AddressingMode::Indirect_Y => {
            let base = args[0];
            let deref_base = cpu.mem_read_u16(base as u16);
            let deref = deref_base.wrapping_add(cpu.register_y as u16);
            let value = cpu.mem_read(deref);
            format!("= {:<04X} @ {:<04X} = {:<02X}", deref_base, deref, value)
        }
        _ => {
            format!("")
        }
    }
}

fn cpu2str(cpu: &CPU) -> String {
    format!(
        "A:{:<02X} X:{:<02X} Y:{:<02X} P:{:<02X} SP:{:<02X}",
        cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer,
    )
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::bus::Bus;
    use crate::cartridge::test::test_rom;
    use crate::ppu::NesPPU;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom(), move |ppu: &NesPPU| {});
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom(), move |ppu: &NesPPU| {});
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        // data
        bus.mem_write(0x33, 0x00);
        bus.mem_write(0x34, 0x04);

        // target cell
        bus.mem_write(0x0400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;

        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }

    /* Instruction tests
    use super::*;
    fn run<F>(program: Vec<u8>, f: F) -> CPU
    where
        F: Fn(&mut CPU),
    {
        let mut cpu = CPU::new(Rom::empty());
        cpu.load();
        cpu.reset();
        f(&mut cpu);
        cpu.run();
        cpu
    }

    fn assert_status(cpu: &CPU, flags: u8) {
        assert_eq!(cpu.status, flags)
    }

    // LDA
    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let cpu = run(vec![0xa9, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_a, 0x05);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let cpu = run(vec![0xa9, 0x00, 0x00], |_| {});
        assert_status(&cpu, FLAG_ZERO);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let cpu = run(vec![0xa9, 0x80, 0x00], |_| {});
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_lda_from_memory_zero_page() {
        let cpu = run(vec![0xa5, 0x10, 0x00], |cpu| {
            cpu.mem_write(0x10, 0x55);
        });
        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_lda_from_memory_zero_page_x() {
        let cpu = run(vec![0xb5, 0x10, 0x00], |cpu| {
            cpu.mem_write(0x11, 0x56);
            cpu.register_x = 0x01;
        });
        assert_eq!(cpu.register_a, 0x56);
    }

    #[test]
    fn test_lda_from_memory_absolute() {
        let cpu = run(vec![0xad, 0x10, 0xaa, 0x00], |cpu| {
            cpu.mem_write(0xAA10, 0x57);
        });
        assert_eq!(cpu.register_a, 0x57);
    }

    #[test]
    fn test_lda_from_memory_absolute_x() {
        let cpu = run(vec![0xbd, 0x10, 0xaa, 0x00], |cpu| {
            cpu.mem_write(0xAA15, 0x58);
            cpu.register_x = 0x05;
        });
        assert_eq!(cpu.register_a, 0x58);
    }

    #[test]
    fn test_lda_from_memory_absolute_y() {
        let cpu = run(vec![0xb9, 0x10, 0xaa, 0x00], |cpu| {
            cpu.mem_write(0xAA18, 0x59);
            cpu.register_y = 0x08;
        });
        assert_eq!(cpu.register_a, 0x59);
    }

    #[test]
    fn test_lda_from_memory_indirect_x() {
        let cpu = run(vec![0xa1, 0x10, 0x00], |cpu| {
            cpu.mem_write_u16(0x18, 0xFF05);
            cpu.mem_write(0xFF05, 0x5A);
            cpu.register_x = 0x08;
        });
        assert_eq!(cpu.register_a, 0x5A);
    }

    #[test]
    fn test_lda_from_memory_indirect_y() {
        let cpu = run(vec![0xb1, 0x10, 0x00], |cpu| {
            cpu.mem_write_u16(0x10, 0xFF06);
            cpu.mem_write(0xFF09, 0x5B);
            cpu.register_y = 0x03;
        });
        assert_eq!(cpu.register_a, 0x5B);
    }

    #[test]
    fn test_5_ops_working_together() {
        let cpu = run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0xc1);
    }

    // STA
    #[test]
    fn test_sta_from_memory() {
        let cpu = run(vec![0x85, 0x10, 0x00], |cpu| {
            cpu.register_a = 0xBA;
        });
        assert_eq!(cpu.mem_read(0x10), 0xBA);
    }

    // ADC
    #[test]
    fn test_adc_no_carry() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x30);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_adc_has_carry() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x31);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_adc_occur_carry() {
        let cpu = run(vec![0x69, 0x01, 0x00], |cpu| {
            cpu.register_a = 0xFF;
        });
        assert_eq!(cpu.register_a, 0x00);
        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
    }

    #[test]
    fn test_adc_occur_overflow_plus() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x7F;
        });
        assert_eq!(cpu.register_a, 0x8F);
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_adc_occur_overflow_plus_with_carry() {
        let cpu = run(vec![0x69, 0x6F, 0x00], |cpu| {
            cpu.register_a = 0x10;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_adc_occur_overflow_minus() {
        let cpu = run(vec![0x69, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(&cpu, FLAG_OVERFLOW | FLAG_CARRY);
    }

    #[test]
    fn test_adc_occur_overflow_minus_with_carry() {
        let mut cpu = run(vec![0x69, 0x80, 0x00], |cpu| {
            cpu.register_a = 0x80;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_OVERFLOW | FLAG_CARRY);
    }

    #[test]
    fn test_adc_no_overflow() {
        let cpu = run(vec![0x69, 0x7F, 0x00], |cpu| {
            cpu.register_a = 0x82;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    // SBC
    #[test]
    fn test_sbc_no_carry() {
        let cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x0F);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_sbc_has_carry() {
        let mut cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x10);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_sbc_occur_carry() {
        let cpu = run(vec![0xe9, 0x02, 0x00], |cpu| {
            cpu.register_a = 0x01;
        });
        assert_eq!(cpu.register_a, 0xFE);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_sbc_occur_overflow() {
        let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x7F;
        });
        assert_eq!(cpu.register_a, 0xFD);
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_sbc_occur_overflow_with_carry() {
        let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x7F;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xFE);
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_sbc_no_overflow() {
        let cpu = run(vec![0xe9, 0x7F, 0x00], |cpu| {
            cpu.register_a = 0x7E;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xFF);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // AND
    #[test]
    fn test_and() {
        let cpu = run(vec![0x29, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x08);
        assert_status(&cpu, 0);
    }

    // EOR
    #[test]
    fn test_eor() {
        let cpu = run(vec![0x49, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x06);
        assert_status(&cpu, 0);
    }

    // ORA
    #[test]
    fn test_ora() {
        let cpu = run(vec![0x09, 0x0C, 0x00], |cpu| {
            cpu.register_a = 0x0A;
        });
        assert_eq!(cpu.register_a, 0x0E);
        assert_status(&cpu, 0);
    }

    // ASL
    #[test]
    fn test_asl_a() {
        let cpu = run(vec![0x0A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_asl_zero_page() {
        let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_asl_a_occur_carry() {
        let cpu = run(vec![0x0A, 0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_asl_zero_page_occur_carry() {
        let cpu = run(vec![0x06, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x81);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x02);
        assert_status(&cpu, FLAG_CARRY);
    }

    // LSR
    #[test]
    fn test_lsr_a() {
        let cpu = run(vec![0x4A, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_zero_page() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_lsr_zero_page_zero_flag() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x01);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x00);
        assert_status(&cpu, FLAG_ZERO | FLAG_CARRY);
    }

    #[test]
    fn test_lsr_a_occur_carry() {
        let cpu = run(vec![0x4A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_lsr_zero_page_occur_carry() {
        let cpu = run(vec![0x46, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    // ROL
    #[test]
    fn test_rol_a() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_a_with_carry() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_with_carry() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x03 * 2 + 1);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_a_zero_with_carry() {
        let cpu = run(vec![0x2A, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_rol_zero_page_zero_with_carry() {
        let cpu = run(vec![0x26, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    // ROR
    #[test]
    fn test_ror_a() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_zero_page() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x02);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_ror_a_occur_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x03;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_zero_page_occur_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x01);
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_ror_a_with_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x03;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_with_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x03);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x81);
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_a_zero_with_carry() {
        let cpu = run(vec![0x6A, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_ror_zero_page_zero_with_carry() {
        let cpu = run(vec![0x66, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.mem_read(0x0001), 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // BCC
    #[test]
    fn test_bcc() {
        let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    #[test]
    fn test_bcc_with_carry() {
        let cpu = run(vec![0x90, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_CARRY);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    #[test]
    fn test_bcc_negative() {
        let cpu = run(vec![0x90, 0xfc, 0x00], |cpu| {
            cpu.mem_write(0x7FFF, 0x00);
            cpu.mem_write(0x7FFE, 0xe8);
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8000)
    }

    // BCS
    #[test]
    fn test_bcs() {
        let cpu = run(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    #[test]
    fn test_bcs_with_carry() {
        let cpu = run(vec![0xb0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, FLAG_CARRY);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    #[test]
    fn test_bcs_negative() {
        let cpu = run(vec![0xb0, 0xfc, 0x00], |cpu| {
            cpu.mem_write(0x7FFF, 0x00);
            cpu.mem_write(0x7FFE, 0xe8);
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, FLAG_CARRY);
        assert_eq!(cpu.program_counter, 0x8000)
    }

    // BEQ
    #[test]
    fn test_beq() {
        let cpu = run(vec![0xF0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    #[test]
    fn test_beq_with_zero_flag() {
        let cpu = run(vec![0xF0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_ZERO;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0); // ZEROはINXで落ちる
        assert_eq!(cpu.program_counter, 0x8006)
    }

    // BNE
    #[test]
    fn test_bne() {
        let cpu = run(vec![0xD0, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    #[test]
    fn test_bne_with_zero_flag() {
        let cpu = run(vec![0xD0, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_ZERO;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_ZERO);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    // BIT
    #[test]
    fn test_bit() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x0000, 0x00);
        });
        assert_status(&cpu, FLAG_ZERO);
    }

    #[test]
    fn test_bit_negative_flag() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x00;
            cpu.mem_write(0x0000, 0x80);
        });
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_ZERO);
    }

    #[test]
    fn test_bit_overflow_flag() {
        let cpu = run(vec![0x24, 0x00, 0x00], |cpu| {
            cpu.register_a = 0x40;
            cpu.mem_write(0x0000, 0x40);
        });
        assert_status(&cpu, FLAG_OVERFLOW);
    }

    // BMI
    #[test]
    fn test_bmi() {
        let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    #[test]
    fn test_bmi_with_negative_flag() {
        let cpu = run(vec![0x30, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0); //INXしてるからnegativeが落ちる
        assert_eq!(cpu.program_counter, 0x8006)
    }

    // BPL
    #[test]
    fn test_bpl() {
        let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    #[test]
    fn test_bpl_with_negative_flag() {
        let cpu = run(vec![0x10, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_NEGATIVE);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    // BVC
    #[test]
    fn test_bvc() {
        let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    #[test]
    fn test_bvc_with_overflow_flag() {
        let cpu = run(vec![0x50, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_OVERFLOW);
        assert_eq!(cpu.program_counter, 0x8003)
    }

    // BVS
    #[test]
    fn test_bvs() {
        let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8003);
    }

    #[test]
    fn test_bvs_with_overflow_flag() {
        let cpu = run(vec![0x70, 0x02, 0x00, 0x00, 0xe8, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, FLAG_OVERFLOW);
        assert_eq!(cpu.program_counter, 0x8006)
    }

    // CLC
    #[test]
    fn test_clc() {
        let cpu = run(vec![0x18, 0x00], |cpu| {
            cpu.status = FLAG_CARRY | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SEC
    #[test]
    fn test_sec() {
        let cpu = run(vec![0x38, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_CARRY | FLAG_NEGATIVE);
    }

    // CLD
    #[test]
    fn test_cld() {
        let cpu = run(vec![0xd8, 0x00], |cpu| {
            cpu.status = FLAG_DECIMAL | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SED
    #[test]
    fn test_sed() {
        let cpu = run(vec![0xf8, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_DECIMAL | FLAG_NEGATIVE);
    }

    // CLI
    #[test]
    fn test_cli() {
        let cpu = run(vec![0x58, 0x00], |cpu| {
            cpu.status = FLAG_INTERRRUPT | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // SEI
    #[test]
    fn test_sei() {
        let cpu = run(vec![0x78, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_INTERRRUPT | FLAG_NEGATIVE);
    }

    // CLV
    #[test]
    fn test_clv() {
        let cpu = run(vec![0xb8, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW | FLAG_NEGATIVE;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // CMP
    #[test]
    fn test_cmp() {
        let cpu = run(vec![0xC9, 0x01, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY);
    }

    #[test]
    fn test_cmp_eq() {
        let cpu = run(vec![0xC9, 0x02, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
    }

    #[test]
    fn test_cmp_negative() {
        let cpu = run(vec![0xC9, 0x03, 0x00], |cpu| {
            cpu.register_a = 0x02;
        });
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // CPX
    #[test]
    fn test_cpx() {
        let cpu = run(vec![0xe0, 0x01, 0x00], |cpu| {
            cpu.register_x = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY);
    }

    // CPY
    #[test]
    fn test_cpy() {
        let cpu = run(vec![0xc0, 0x01, 0x00], |cpu| {
            cpu.register_y = 0x02;
        });
        assert_status(&cpu, FLAG_CARRY);
    }

    // DEC
    #[test]
    fn test_dec() {
        let cpu = run(vec![0xc6, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x05);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x04);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_dec_overflow() {
        let cpu = run(vec![0xc6, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x00);
        });
        assert_eq!(cpu.mem_read(0x0001), 0xFF);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // DEX
    #[test]
    fn test_dex() {
        let cpu = run(vec![0xca, 0x00], |cpu| {
            cpu.register_x = 0x05;
        });
        assert_eq!(cpu.register_x, 0x04);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_dex_overflow() {
        let cpu = run(vec![0xca, 0x00], |cpu| {
            cpu.register_x = 0x00;
        });
        assert_eq!(cpu.register_x, 0xFF);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // DEY
    #[test]
    fn test_dey() {
        let cpu = run(vec![0x88, 0x00], |cpu| {
            cpu.register_y = 0x05;
        });
        assert_eq!(cpu.register_y, 0x04);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_dey_overflow() {
        let cpu = run(vec![0x88, 0x00], |cpu| {
            cpu.register_y = 0x00;
        });
        assert_eq!(cpu.register_y, 0xFF);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    // INC
    #[test]
    fn test_inc() {
        let cpu = run(vec![0xe6, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0x05);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_inc_overflow() {
        let cpu = run(vec![0xe6, 0x01, 0x00], |cpu| {
            cpu.mem_write(0x0001, 0xFF);
        });
        assert_eq!(cpu.mem_read(0x0001), 0x00);
        assert_status(&cpu, FLAG_ZERO);
    }

    // INX
    #[test]
    fn test_inx() {
        let cpu = run(vec![0xe8, 0x00], |cpu| {
            cpu.register_x = 0x05;
        });
        assert_eq!(cpu.register_x, 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_inx_overflow() {
        let cpu = run(vec![0xe8, 0x00], |cpu| {
            cpu.register_x = 0xFF;
        });
        assert_eq!(cpu.register_x, 0x00);
        assert_status(&cpu, FLAG_ZERO);
    }

    // INY
    #[test]
    fn test_iny() {
        let cpu = run(vec![0xc8, 0x00], |cpu| {
            cpu.register_y = 0x05;
        });
        assert_eq!(cpu.register_y, 0x06);
        assert_status(&cpu, 0);
    }

    #[test]
    fn test_iny_overflow() {
        let cpu = run(vec![0xc8, 0x00], |cpu| {
            cpu.register_y = 0xFF;
        });
        assert_eq!(cpu.register_y, 0x00);
        assert_status(&cpu, FLAG_ZERO);
    }

    // JMP
    #[test]
    fn test_jmp() {
        let cpu = run(vec![0x4c, 0x30, 0x40, 0x00], |cpu| {
            cpu.mem_write(0x4030, 0xe8);
            cpu.mem_write(0x4031, 0x00);
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x4032);
    }

    #[test]
    fn test_jmp_indirect() {
        let cpu = run(vec![0x6c, 0x30, 0x40, 0x00], |cpu| {
            cpu.mem_write(0x4030, 0x01);
            cpu.mem_write(0x4031, 0x02);

            cpu.mem_write(0x0201, 0xe8);
            cpu.mem_write(0x0202, 0x00);
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x0203);
    }

    // JSR
    #[test]
    fn test_jsr() {
        let cpu = run(vec![0x20, 0x30, 0x40, 0x00], |cpu| {
            cpu.mem_write(0x4030, 0xe8);
            cpu.mem_write(0x4031, 0x00);
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x4032);
        assert_eq!(cpu.stack_pointer, 0xFD);
        assert_eq!(cpu.mem_read_u16(0x01FE), 0x8003);
    }

    // RTS
    #[test]
    fn test_rts() {
        let cpu = run(vec![0x60, 0x00], |cpu| {
            cpu.mem_write(0x01FF, 0x05);
            cpu.mem_write(0x01FE, 0x06);

            cpu.mem_write(0x0506, 0xe8);
            cpu.mem_write(0x0507, 0x00);

            cpu.stack_pointer = 0xFD;
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x0508);
        assert_eq!(cpu.stack_pointer, 0xFF);
        // 書き潰されない前提
        assert_eq!(cpu.mem_read_u16(0x01FE), 0x0506);
    }

    // JSR & RTS
    #[test]
    fn test_jsr_and_rts() {
        let cpu = run(vec![0x20, 0x30, 0x40, 0x00], |cpu| {
            cpu.mem_write(0x4030, 0xe8);
            cpu.mem_write(0x4031, 0x60); // RTS
            cpu.mem_write(0x4032, 0x00);
        });
        assert_eq!(cpu.register_x, 0x01);
        assert_status(&cpu, 0);
        assert_eq!(cpu.program_counter, 0x8004);
        assert_eq!(cpu.stack_pointer, 0xFF);
        // 書き潰されない前提
        assert_eq!(cpu.mem_read_u16(0x01FE), 0x8003);
    }

    // LDX
    #[test]
    fn test_ldx() {
        let cpu = run(vec![0xa2, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0x05);
        assert_status(&cpu, 0);
    }

    // LDY
    #[test]
    fn test_ldy() {
        let cpu = run(vec![0xa0, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_y, 0x05);
        assert_status(&cpu, 0);
    }

    // NOP
    #[test]
    fn test_nop() {
        let cpu = run(vec![0xea, 0x00], |_| {});
        assert_eq!(cpu.program_counter, 0x8002);
        assert_status(&cpu, 0);
    }

    // PHA
    #[test]
    fn test_pha() {
        let cpu = run(vec![0x48, 0x00], |cpu| {
            cpu.register_a = 0x07;
        });
        assert_status(&cpu, 0);
        assert_eq!(cpu.register_a, 0x07);
        assert_eq!(cpu.stack_pointer, 0xFE);
        assert_eq!(cpu.mem_read(0x01FF), 0x07);
    }

    // PLA
    #[test]
    fn test_pla() {
        let cpu = run(vec![0x68, 0x00], |cpu| {
            cpu.mem_write(0x01FF, 0x07);
            cpu.stack_pointer = 0xFE;
        });
        assert_eq!(cpu.register_a, 0x07);
        assert_status(&cpu, 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_pla_zero() {
        let cpu = run(vec![0x68, 0x00], |cpu| {
            cpu.mem_write(0x01FF, 0x00);
            cpu.stack_pointer = 0xFE;
        });
        assert_eq!(cpu.register_a, 0x00);
        assert_status(&cpu, FLAG_ZERO);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    // PHA & PLA
    #[test]
    fn test_pla_and_pla() {
        let cpu = run(vec![0x48, 0xa9, 0x60, 0x68, 0x00], |cpu| {
            cpu.register_a = 0x80;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(&cpu, FLAG_NEGATIVE);
        assert_eq!(cpu.stack_pointer, 0xFF);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    // PHP
    #[test]
    fn test_php() {
        let cpu = run(vec![0x08, 0x00], |cpu| {
            cpu.status = FLAG_NEGATIVE | FLAG_OVERFLOW;
        });
        assert_status(&cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
        assert_eq!(cpu.stack_pointer, 0xFE);
        assert_eq!(cpu.mem_read(0x01FF), FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    // PLP
    #[test]
    fn test_plp() {
        let cpu = run(vec![0x28, 0x00], |cpu| {
            cpu.mem_write(0x01FF, FLAG_CARRY | FLAG_ZERO);
            cpu.stack_pointer = 0xFE;
        });
        assert_status(&cpu, FLAG_CARRY | FLAG_ZERO);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    // PHP & PLP
    #[test]
    fn test_plp_and_plp() {
        let cpu = run(vec![0x08, 0xa9, 0xF0, 0x28, 0x00], |cpu| {
            cpu.status = FLAG_OVERFLOW | FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xF0);
        assert_status(&cpu, FLAG_OVERFLOW | FLAG_CARRY);
        assert_eq!(cpu.stack_pointer, 0xFF);
        assert_eq!(cpu.program_counter, 0x8005);
    }

    // FIXME RTIのテストは一旦保留
    // BRKの逆をやるのだが、BRKでpushしてないぽいので。

    // STX
    #[test]
    fn test_stx() {
        let cpu = run(vec![0x86, 0x10, 0x00], |cpu| {
            cpu.register_x = 0xBA;
        });
        assert_eq!(cpu.mem_read(0x10), 0xBA);
    }

    // STY
    #[test]
    fn test_sty() {
        let cpu = run(vec![0x84, 0x10, 0x00], |cpu| {
            cpu.register_y = 0xBA;
        });
        assert_eq!(cpu.mem_read(0x10), 0xBA);
    }

    // TAX
    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let cpu = run(vec![0xaa, 0x00], |cpu| {
            cpu.register_a = 10;
        });
        assert_eq!(cpu.register_x, 10);
    }

    // TXA
    #[test]
    fn test_txa() {
        let cpu = run(vec![0x8a, 0x00], |cpu| {
            cpu.register_x = 0x10;
        });
        assert_eq!(cpu.register_a, 0x10);
    }

    // TAY
    #[test]
    fn test_tay() {
        let cpu = run(vec![0xa8, 0x00], |cpu| {
            cpu.register_a = 0x10;
        });
        assert_eq!(cpu.register_y, 0x10);
    }

    // TYA
    #[test]
    fn test_tya() {
        let cpu = run(vec![0x98, 0x00], |cpu| {
            cpu.register_y = 0x10;
        });
        assert_eq!(cpu.register_a, 0x10);
    }

    // TSX
    #[test]
    fn test_tsx() {
        let cpu = run(vec![0xba, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0xFF);
        assert_status(&cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_tsx_some_value() {
        let cpu = run(vec![0xba, 0x00], |cpu| {
            cpu.stack_pointer = 0x75;
        });
        assert_eq!(cpu.register_x, 0x75);
        assert_status(&cpu, 0);
    }

    // TXS
    #[test]
    fn test_txs() {
        let cpu = run(vec![0x9a, 0x00], |cpu| {
            cpu.register_x = 0x80;
        });
        assert_eq!(cpu.stack_pointer, 0x80);
        assert_status(&cpu, 0);
    }
    */
}
