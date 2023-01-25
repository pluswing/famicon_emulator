fn main() {
    println!("Hello, world!");
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

const FLAG_CARRY: u8 = 1 << 0;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_INTERRRUPT: u8 = 1 << 2;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_BREAK: u8 = 1 << 4;
// 5 は未使用。
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_NEGATIVE: u8 = 1 << 7;

const SIGN_BIT: u8 = 1 << 7;

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub program_counter: u16,
    memory: [u8; 0x10000], // 0xFFFF
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0x00; 0x10000],
        }
    }

    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
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
                addr
            }

            // LDA $4400,Y => b9 00 44
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(self.program_counter);
                let addr = base.wrapping_add(self.register_y as u16);
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
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0x00FF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = 0;
        // TODO memoryリセット必要？？

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            println!("OPS: {:X}", opscode);

            match opscode {
                0xE9 => {
                    self.sbc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0x69 => {
                    self.adc(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                /* LDA */
                0xA9 => {
                    self.lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBD => {
                    self.lda(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }

                0x00 => return,
                0xAA => self.tax(),
                0xE8 => self.inx(),

                /* STA */
                0x85 => {
                    self.sta(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x95 => {
                    self.sta(&AddressingMode::ZeroPage_X);
                    self.program_counter += 1;
                }
                0x8D => {
                    self.sta(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x9D => {
                    self.sta(&AddressingMode::Absolute_X);
                    self.program_counter += 2;
                }
                0x99 => {
                    self.sta(&AddressingMode::Absolute_Y);
                    self.program_counter += 2;
                }
                0x81 => {
                    self.sta(&AddressingMode::Indirect_X);
                    self.program_counter += 1;
                }
                0x91 => {
                    self.sta(&AddressingMode::Indirect_Y);
                    self.program_counter += 1;
                }
                _ => todo!(""),
            }
        }
    }

    fn sbc(&mut self, mode: &AddressingMode) {
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

    fn adc(&mut self, mode: &AddressingMode) {
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

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
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

#[cfg(test)]
mod test {
    use super::*;

    fn run<F>(program: Vec<u8>, f: F) -> CPU
    where
        F: Fn(&mut CPU),
    {
        let mut cpu = CPU::new();
        cpu.load(program);
        cpu.reset();
        f(&mut cpu);
        cpu.run();
        cpu
    }

    fn assert_status(cpu: CPU, flags: u8) {
        assert_eq!(cpu.status, flags)
    }

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let cpu = run(vec![0xa9, 0x05, 0x00], |_| {});
        assert_eq!(cpu.register_a, 0x05);
        assert_status(cpu, 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let cpu = run(vec![0xa9, 0x00, 0x00], |_| {});
        assert_status(cpu, FLAG_ZERO);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let cpu = run(vec![0xa9, 0x80, 0x00], |_| {});
        assert_status(cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let cpu = run(vec![0xaa, 0x00], |cpu| {
            cpu.register_a = 10;
        });
        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let cpu = run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00], |_| {});
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let cpu = run(vec![0xe8, 0xe8, 0x00], |cpu| {
            cpu.register_x = 0xff;
        });
        assert_eq!(cpu.register_x, 1);
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
    fn test_sta_from_memory() {
        let cpu = run(vec![0x85, 0x10, 0x00], |cpu| {
            cpu.register_a = 0xBA;
        });
        assert_eq!(cpu.mem_read(0x10), 0xBA);
    }

    #[test]
    fn test_adc_no_carry() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x30);
        assert_status(cpu, 0);
    }

    #[test]
    fn test_adc_has_carry() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x31);
        assert_status(cpu, 0);
    }

    #[test]
    fn test_adc_occur_carry() {
        let cpu = run(vec![0x69, 0x01, 0x00], |cpu| {
            cpu.register_a = 0xFF;
        });
        assert_eq!(cpu.register_a, 0x00);
        assert_status(cpu, FLAG_CARRY | FLAG_ZERO);
    }

    #[test]
    fn test_adc_occur_overflow_plus() {
        let cpu = run(vec![0x69, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x7F;
        });
        assert_eq!(cpu.register_a, 0x8F);
        assert_status(cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_adc_occur_overflow_plus_with_carry() {
        let cpu = run(vec![0x69, 0x6F, 0x00], |cpu| {
            cpu.register_a = 0x10;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x80);
        assert_status(cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_adc_occur_overflow_minus() {
        let cpu = run(vec![0x69, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x81;
        });
        assert_eq!(cpu.register_a, 0x02);
        assert_status(cpu, FLAG_OVERFLOW | FLAG_CARRY);
    }

    #[test]
    fn test_adc_occur_overflow_minus_with_carry() {
        let mut cpu = run(vec![0x69, 0x80, 0x00], |cpu| {
            cpu.register_a = 0x80;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(cpu, FLAG_OVERFLOW | FLAG_CARRY);
    }

    #[test]
    fn test_adc_no_overflow() {
        let cpu = run(vec![0x69, 0x7F, 0x00], |cpu| {
            cpu.register_a = 0x82;
        });
        assert_eq!(cpu.register_a, 0x01);
        assert_status(cpu, FLAG_CARRY);
    }

    #[test]
    fn test_sbc_no_carry() {
        let cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
        });
        assert_eq!(cpu.register_a, 0x0F);
        assert_status(cpu, FLAG_CARRY);
    }

    #[test]
    fn test_sbc_has_carry() {
        let mut cpu = run(vec![0xe9, 0x10, 0x00], |cpu| {
            cpu.register_a = 0x20;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0x10);
        assert_status(cpu, FLAG_CARRY);
    }

    #[test]
    fn test_sbc_occur_carry() {
        let cpu = run(vec![0xe9, 0x02, 0x00], |cpu| {
            cpu.register_a = 0x01;
        });
        assert_eq!(cpu.register_a, 0xFE);
        assert_status(cpu, FLAG_NEGATIVE);
    }

    #[test]
    fn test_sbc_occur_overflow() {
        let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x7F;
        });
        assert_eq!(cpu.register_a, 0xFD);
        assert_status(cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_sbc_occur_overflow_with_carry() {
        let cpu = run(vec![0xe9, 0x81, 0x00], |cpu| {
            cpu.register_a = 0x7F;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xFE);
        assert_status(cpu, FLAG_NEGATIVE | FLAG_OVERFLOW);
    }

    #[test]
    fn test_sbc_no_overflow() {
        let cpu = run(vec![0xe9, 0x7F, 0x00], |cpu| {
            cpu.register_a = 0x7E;
            cpu.status = FLAG_CARRY;
        });
        assert_eq!(cpu.register_a, 0xFF);
        assert_status(cpu, FLAG_NEGATIVE);
    }
}
