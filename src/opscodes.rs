use std::{collections::HashMap, sync::Mutex};
use once_cell::sync::Lazy;
use crate::cpu::{AddressingMode, CycleCalcMode, OpCode, CPU};

pub static CPU_OPS_CODES: Lazy<HashMap<u8, OpCode>> = Lazy::new(|| {
  let mut m = HashMap::new();

  m.insert(0x69, OpCode::new(0x69, "ADC", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x65, OpCode::new(0x65, "ADC", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x75, OpCode::new(0x75, "ADC", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x6D, OpCode::new(0x6D, "ADC", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x7D, OpCode::new(0x7D, "ADC", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x79, OpCode::new(0x79, "ADC", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0x61, OpCode::new(0x61, "ADC", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x71, OpCode::new(0x71, "ADC", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0x29, OpCode::new(0x29, "AND", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x25, OpCode::new(0x25, "AND", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x35, OpCode::new(0x35, "AND", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x2D, OpCode::new(0x2D, "AND", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x3D, OpCode::new(0x3D, "AND", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x39, OpCode::new(0x39, "AND", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0x21, OpCode::new(0x21, "AND", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x31, OpCode::new(0x31, "AND", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0x0A, OpCode::new(0x0A, "ASL", 1, 2, CycleCalcMode::None, AddressingMode::Accumulator));
  m.insert(0x06, OpCode::new(0x06, "ASL", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x16, OpCode::new(0x16, "ASL", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x0E, OpCode::new(0x0E, "ASL", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x1E, OpCode::new(0x1E, "ASL", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x90, OpCode::new(0x90, "BCC", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0xB0, OpCode::new(0xB0, "BCS", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0xF0, OpCode::new(0xF0, "BEQ", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0x24, OpCode::new(0x24, "BIT", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x2C, OpCode::new(0x2C, "BIT", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x30, OpCode::new(0x30, "BMI", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0xD0, OpCode::new(0xD0, "BNE", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0x10, OpCode::new(0x10, "BPL", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0x00, OpCode::new(0x00, "BRK", 1, 7, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x50, OpCode::new(0x50, "BVC", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0x70, OpCode::new(0x70, "BVS", 2, 2, CycleCalcMode::Branch, AddressingMode::Relative));
  m.insert(0x18, OpCode::new(0x18, "CLC", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xD8, OpCode::new(0xD8, "CLD", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x58, OpCode::new(0x58, "CLI", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xB8, OpCode::new(0xB8, "CLV", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xC9, OpCode::new(0xC9, "CMP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xC5, OpCode::new(0xC5, "CMP", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xD5, OpCode::new(0xD5, "CMP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xCD, OpCode::new(0xCD, "CMP", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xDD, OpCode::new(0xDD, "CMP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0xD9, OpCode::new(0xD9, "CMP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xC1, OpCode::new(0xC1, "CMP", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xD1, OpCode::new(0xD1, "CMP", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0xE0, OpCode::new(0xE0, "CPX", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xE4, OpCode::new(0xE4, "CPX", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xEC, OpCode::new(0xEC, "CPX", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xC0, OpCode::new(0xC0, "CPY", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xC4, OpCode::new(0xC4, "CPY", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xCC, OpCode::new(0xCC, "CPY", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xC6, OpCode::new(0xC6, "DEC", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xD6, OpCode::new(0xD6, "DEC", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xCE, OpCode::new(0xCE, "DEC", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xDE, OpCode::new(0xDE, "DEC", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0xCA, OpCode::new(0xCA, "DEX", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x88, OpCode::new(0x88, "DEY", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x49, OpCode::new(0x49, "EOR", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x45, OpCode::new(0x45, "EOR", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x55, OpCode::new(0x55, "EOR", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x4D, OpCode::new(0x4D, "EOR", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x5D, OpCode::new(0x5D, "EOR", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x59, OpCode::new(0x59, "EOR", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0x41, OpCode::new(0x41, "EOR", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x51, OpCode::new(0x51, "EOR", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0xE6, OpCode::new(0xE6, "INC", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xF6, OpCode::new(0xF6, "INC", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xEE, OpCode::new(0xEE, "INC", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xFE, OpCode::new(0xFE, "INC", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0xE8, OpCode::new(0xE8, "INX", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xC8, OpCode::new(0xC8, "INY", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x4C, OpCode::new(0x4C, "JMP", 3, 3, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x6C, OpCode::new(0x6C, "JMP", 3, 5, CycleCalcMode::None, AddressingMode::Indirect));
  m.insert(0x20, OpCode::new(0x20, "JSR", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xA9, OpCode::new(0xA9, "LDA", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xA5, OpCode::new(0xA5, "LDA", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xB5, OpCode::new(0xB5, "LDA", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xAD, OpCode::new(0xAD, "LDA", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xBD, OpCode::new(0xBD, "LDA", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0xB9, OpCode::new(0xB9, "LDA", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xA1, OpCode::new(0xA1, "LDA", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xB1, OpCode::new(0xB1, "LDA", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0xA2, OpCode::new(0xA2, "LDX", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xA6, OpCode::new(0xA6, "LDX", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xB6, OpCode::new(0xB6, "LDX", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_Y));
  m.insert(0xAE, OpCode::new(0xAE, "LDX", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xBE, OpCode::new(0xBE, "LDX", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xA0, OpCode::new(0xA0, "LDY", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xA4, OpCode::new(0xA4, "LDY", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xB4, OpCode::new(0xB4, "LDY", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xAC, OpCode::new(0xAC, "LDY", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xBC, OpCode::new(0xBC, "LDY", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x4A, OpCode::new(0x4A, "LSR", 1, 2, CycleCalcMode::None, AddressingMode::Accumulator));
  m.insert(0x46, OpCode::new(0x46, "LSR", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x56, OpCode::new(0x56, "LSR", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x4E, OpCode::new(0x4E, "LSR", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x5E, OpCode::new(0x5E, "LSR", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0xEA, OpCode::new(0xEA, "NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x09, OpCode::new(0x09, "ORA", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x05, OpCode::new(0x05, "ORA", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x15, OpCode::new(0x15, "ORA", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x0D, OpCode::new(0x0D, "ORA", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x1D, OpCode::new(0x1D, "ORA", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x19, OpCode::new(0x19, "ORA", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0x01, OpCode::new(0x01, "ORA", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x11, OpCode::new(0x11, "ORA", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0x48, OpCode::new(0x48, "PHA", 1, 3, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x08, OpCode::new(0x08, "PHP", 1, 3, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x68, OpCode::new(0x68, "PLA", 1, 4, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x28, OpCode::new(0x28, "PLP", 1, 4, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x2A, OpCode::new(0x2A, "ROL", 1, 2, CycleCalcMode::None, AddressingMode::Accumulator));
  m.insert(0x26, OpCode::new(0x26, "ROL", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x36, OpCode::new(0x36, "ROL", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x2E, OpCode::new(0x2E, "ROL", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x3E, OpCode::new(0x3E, "ROL", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x6A, OpCode::new(0x6A, "ROR", 1, 2, CycleCalcMode::None, AddressingMode::Accumulator));
  m.insert(0x66, OpCode::new(0x66, "ROR", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x76, OpCode::new(0x76, "ROR", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x6E, OpCode::new(0x6E, "ROR", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x7E, OpCode::new(0x7E, "ROR", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x40, OpCode::new(0x40, "RTI", 1, 6, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x60, OpCode::new(0x60, "RTS", 1, 6, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xE9, OpCode::new(0xE9, "SBC", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xE5, OpCode::new(0xE5, "SBC", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xF5, OpCode::new(0xF5, "SBC", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xED, OpCode::new(0xED, "SBC", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xFD, OpCode::new(0xFD, "SBC", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0xF9, OpCode::new(0xF9, "SBC", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xE1, OpCode::new(0xE1, "SBC", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xF1, OpCode::new(0xF1, "SBC", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0x38, OpCode::new(0x38, "SEC", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xF8, OpCode::new(0xF8, "SED", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x78, OpCode::new(0x78, "SEI", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x85, OpCode::new(0x85, "STA", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x95, OpCode::new(0x95, "STA", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x8D, OpCode::new(0x8D, "STA", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x9D, OpCode::new(0x9D, "STA", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x99, OpCode::new(0x99, "STA", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x81, OpCode::new(0x81, "STA", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x91, OpCode::new(0x91, "STA", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x86, OpCode::new(0x86, "STX", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x96, OpCode::new(0x96, "STX", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_Y));
  m.insert(0x8E, OpCode::new(0x8E, "STX", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x84, OpCode::new(0x84, "STY", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x94, OpCode::new(0x94, "STY", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x8C, OpCode::new(0x8C, "STY", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xAA, OpCode::new(0xAA, "TAX", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xA8, OpCode::new(0xA8, "TAY", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xBA, OpCode::new(0xBA, "TSX", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x8A, OpCode::new(0x8A, "TXA", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x9A, OpCode::new(0x9A, "TXS", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x98, OpCode::new(0x98, "TYA", 1, 2, CycleCalcMode::None, AddressingMode::Implied));

  m.insert(0x0B, OpCode::new(0x0B, "*ANC", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x2B, OpCode::new(0x2B, "*ANC", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x87, OpCode::new(0x87, "*SAX", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x97, OpCode::new(0x97, "*SAX", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_Y));
  m.insert(0x83, OpCode::new(0x83, "*SAX", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x8F, OpCode::new(0x8F, "*SAX", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x6B, OpCode::new(0x6B, "*ARR", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x4B, OpCode::new(0x4B, "*ASR", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xAB, OpCode::new(0xAB, "*LXA", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x9F, OpCode::new(0x9F, "*SHA", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x93, OpCode::new(0x93, "*SHA", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0xCB, OpCode::new(0xCB, "*SBX", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xC7, OpCode::new(0xC7, "*DCP", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xD7, OpCode::new(0xD7, "*DCP", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xCF, OpCode::new(0xCF, "*DCP", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xDF, OpCode::new(0xDF, "*DCP", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0xDB, OpCode::new(0xDB, "*DCP", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0xC3, OpCode::new(0xC3, "*DCP", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xD3, OpCode::new(0xD3, "*DCP", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x04, OpCode::new(0x04, "*NOP", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x14, OpCode::new(0x14, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x34, OpCode::new(0x34, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x44, OpCode::new(0x44, "*NOP", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x54, OpCode::new(0x54, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x64, OpCode::new(0x64, "*NOP", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x74, OpCode::new(0x74, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x80, OpCode::new(0x80, "*NOP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x82, OpCode::new(0x82, "*NOP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x89, OpCode::new(0x89, "*NOP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xC2, OpCode::new(0xC2, "*NOP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xD4, OpCode::new(0xD4, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xE2, OpCode::new(0xE2, "*NOP", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0xF4, OpCode::new(0xF4, "*NOP", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xE7, OpCode::new(0xE7, "*ISB", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xF7, OpCode::new(0xF7, "*ISB", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0xEF, OpCode::new(0xEF, "*ISB", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xFF, OpCode::new(0xFF, "*ISB", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0xFB, OpCode::new(0xFB, "*ISB", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0xE3, OpCode::new(0xE3, "*ISB", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xF3, OpCode::new(0xF3, "*ISB", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x02, OpCode::new(0x02, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x12, OpCode::new(0x12, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x22, OpCode::new(0x22, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x32, OpCode::new(0x32, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x42, OpCode::new(0x42, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x52, OpCode::new(0x52, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x62, OpCode::new(0x62, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x72, OpCode::new(0x72, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x92, OpCode::new(0x92, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xB2, OpCode::new(0xB2, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xD2, OpCode::new(0xD2, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xF2, OpCode::new(0xF2, "*JAM", 1, 0, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xBB, OpCode::new(0xBB, "*LAE", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xA7, OpCode::new(0xA7, "*LAX", 2, 3, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0xB7, OpCode::new(0xB7, "*LAX", 2, 4, CycleCalcMode::None, AddressingMode::ZeroPage_Y));
  m.insert(0xAF, OpCode::new(0xAF, "*LAX", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0xBF, OpCode::new(0xBF, "*LAX", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_Y));
  m.insert(0xA3, OpCode::new(0xA3, "*LAX", 2, 6, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0xB3, OpCode::new(0xB3, "*LAX", 2, 5, CycleCalcMode::Page, AddressingMode::Indirect_Y));
  m.insert(0x1A, OpCode::new(0x1A, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x3A, OpCode::new(0x3A, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x5A, OpCode::new(0x5A, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x7A, OpCode::new(0x7A, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xDA, OpCode::new(0xDA, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0xFA, OpCode::new(0xFA, "*NOP", 1, 2, CycleCalcMode::None, AddressingMode::Implied));
  m.insert(0x27, OpCode::new(0x27, "*RLA", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x37, OpCode::new(0x37, "*RLA", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x2F, OpCode::new(0x2F, "*RLA", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x3F, OpCode::new(0x3F, "*RLA", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x3B, OpCode::new(0x3B, "*RLA", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x23, OpCode::new(0x23, "*RLA", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x33, OpCode::new(0x33, "*RLA", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x67, OpCode::new(0x67, "*RRA", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x77, OpCode::new(0x77, "*RRA", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x6F, OpCode::new(0x6F, "*RRA", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x7F, OpCode::new(0x7F, "*RRA", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x7B, OpCode::new(0x7B, "*RRA", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x63, OpCode::new(0x63, "*RRA", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x73, OpCode::new(0x73, "*RRA", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0xEB, OpCode::new(0xEB, "*SBC", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x07, OpCode::new(0x07, "*SLO", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x17, OpCode::new(0x17, "*SLO", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x0F, OpCode::new(0x0F, "*SLO", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x1F, OpCode::new(0x1F, "*SLO", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x1B, OpCode::new(0x1B, "*SLO", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x03, OpCode::new(0x03, "*SLO", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x13, OpCode::new(0x13, "*SLO", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x47, OpCode::new(0x47, "*SRE", 2, 5, CycleCalcMode::None, AddressingMode::ZeroPage));
  m.insert(0x57, OpCode::new(0x57, "*SRE", 2, 6, CycleCalcMode::None, AddressingMode::ZeroPage_X));
  m.insert(0x4F, OpCode::new(0x4F, "*SRE", 3, 6, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x5F, OpCode::new(0x5F, "*SRE", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x5B, OpCode::new(0x5B, "*SRE", 3, 7, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x43, OpCode::new(0x43, "*SRE", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_X));
  m.insert(0x53, OpCode::new(0x53, "*SRE", 2, 8, CycleCalcMode::None, AddressingMode::Indirect_Y));
  m.insert(0x9E, OpCode::new(0x9E, "*SHX", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m.insert(0x9C, OpCode::new(0x9C, "*SHY", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_X));
  m.insert(0x0C, OpCode::new(0x0C, "*NOP", 3, 4, CycleCalcMode::None, AddressingMode::Absolute));
  m.insert(0x1C, OpCode::new(0x1C, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x3C, OpCode::new(0x3C, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x5C, OpCode::new(0x5C, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x7C, OpCode::new(0x7C, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0xDC, OpCode::new(0xDC, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0xFC, OpCode::new(0xFC, "*NOP", 3, 4, CycleCalcMode::Page, AddressingMode::Absolute_X));
  m.insert(0x8B, OpCode::new(0x8B, "*ANE", 2, 2, CycleCalcMode::None, AddressingMode::Immediate));
  m.insert(0x9B, OpCode::new(0x9B, "*SHS", 3, 5, CycleCalcMode::None, AddressingMode::Absolute_Y));
  m
});


pub fn call(cpu: &mut CPU, op: &OpCode) {
  match op.name.replace("*", "").as_str() {

    "ADC" => {
      cpu.adc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "AND" => {
      cpu.and(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ASL" => {
      cpu.asl(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BCC" => {
      cpu.bcc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BCS" => {
      cpu.bcs(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BEQ" => {
      cpu.beq(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BIT" => {
      cpu.bit(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BMI" => {
      cpu.bmi(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BNE" => {
      cpu.bne(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BPL" => {
      cpu.bpl(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BRK" => {
      cpu.brk(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BVC" => {
      cpu.bvc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "BVS" => {
      cpu.bvs(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CLC" => {
      cpu.clc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CLD" => {
      cpu.cld(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CLI" => {
      cpu.cli(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CLV" => {
      cpu.clv(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CMP" => {
      cpu.cmp(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CPX" => {
      cpu.cpx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "CPY" => {
      cpu.cpy(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "DEC" => {
      cpu.dec(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "DEX" => {
      cpu.dex(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "DEY" => {
      cpu.dey(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "EOR" => {
      cpu.eor(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "INC" => {
      cpu.inc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "INX" => {
      cpu.inx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "INY" => {
      cpu.iny(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "JMP" => {
      cpu.jmp(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "JSR" => {
      cpu.jsr(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LDA" => {
      cpu.lda(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LDX" => {
      cpu.ldx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LDY" => {
      cpu.ldy(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LSR" => {
      cpu.lsr(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "NOP" => {
      cpu.nop(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ORA" => {
      cpu.ora(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "PHA" => {
      cpu.pha(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "PHP" => {
      cpu.php(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "PLA" => {
      cpu.pla(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "PLP" => {
      cpu.plp(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ROL" => {
      cpu.rol(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ROR" => {
      cpu.ror(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "RTI" => {
      cpu.rti(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "RTS" => {
      cpu.rts(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SBC" => {
      cpu.sbc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SEC" => {
      cpu.sec(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SED" => {
      cpu.sed(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SEI" => {
      cpu.sei(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "STA" => {
      cpu.sta(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "STX" => {
      cpu.stx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "STY" => {
      cpu.sty(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TAX" => {
      cpu.tax(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TAY" => {
      cpu.tay(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TSX" => {
      cpu.tsx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TXA" => {
      cpu.txa(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TXS" => {
      cpu.txs(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "TYA" => {
      cpu.tya(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ANC" => {
      cpu.anc(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SAX" => {
      cpu.sax(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ARR" => {
      cpu.arr(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ASR" => {
      cpu.asr(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LXA" => {
      cpu.lxa(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SHA" => {
      cpu.sha(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SBX" => {
      cpu.sbx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "DCP" => {
      cpu.dcp(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ISB" => {
      cpu.isb(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "JAM" => {
      cpu.jam(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LAE" => {
      cpu.lae(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "LAX" => {
      cpu.lax(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "RLA" => {
      cpu.rla(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "RRA" => {
      cpu.rra(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SLO" => {
      cpu.slo(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SRE" => {
      cpu.sre(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SHX" => {
      cpu.shx(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SHY" => {
      cpu.shy(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "ANE" => {
      cpu.ane(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    "SHS" => {
      cpu.shs(&op.addressing_mode);
      cpu.program_counter += op.bytes - 1
    }

    _ => {
        todo!()
    }
  }
}