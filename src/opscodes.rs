use crate::cpu::{AddressingMode, CycleCalcMode, OpCode, CPU};

lazy_static! {
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        OpCode::new(
            0x69,
            "ADC",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x65,
            "ADC",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x75,
            "ADC",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x6D,
            "ADC",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x7D,
            "ADC",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x79,
            "ADC",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x61,
            "ADC",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x71,
            "ADC",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x29,
            "AND",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x25,
            "AND",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x35,
            "AND",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x2D,
            "AND",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x3D,
            "AND",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x39,
            "AND",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x21,
            "AND",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x31,
            "AND",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x0A,
            "ASL",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Accumulator
        ),
        OpCode::new(
            0x06,
            "ASL",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x16,
            "ASL",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x0E,
            "ASL",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x1E,
            "ASL",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x90,
            "BCC",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0xB0,
            "BCS",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0xF0,
            "BEQ",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0x24,
            "BIT",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x2C,
            "BIT",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x30,
            "BMI",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0xD0,
            "BNE",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0x10,
            "BPL",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0x00,
            "BRK",
            1,
            7,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x50,
            "BVC",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0x70,
            "BVS",
            2,
            2,
            CycleCalcMode::Branch,
            AddressingMode::Relative
        ),
        OpCode::new(
            0x18,
            "CLC",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xD8,
            "CLD",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x58,
            "CLI",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xB8,
            "CLV",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xC9,
            "CMP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xC5,
            "CMP",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xD5,
            "CMP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xCD,
            "CMP",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xDD,
            "CMP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xD9,
            "CMP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xC1,
            "CMP",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xD1,
            "CMP",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0xE0,
            "CPX",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xE4,
            "CPX",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xEC,
            "CPX",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xC0,
            "CPY",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xC4,
            "CPY",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xCC,
            "CPY",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xC6,
            "DEC",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xD6,
            "DEC",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xCE,
            "DEC",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xDE,
            "DEC",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xCA,
            "DEX",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x88,
            "DEY",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x49,
            "EOR",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x45,
            "EOR",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x55,
            "EOR",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x4D,
            "EOR",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x5D,
            "EOR",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x59,
            "EOR",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x41,
            "EOR",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x51,
            "EOR",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0xE6,
            "INC",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xF6,
            "INC",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xEE,
            "INC",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xFE,
            "INC",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xE8,
            "INX",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xC8,
            "INY",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x4C,
            "JMP",
            3,
            3,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x6C,
            "JMP",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Indirect
        ),
        OpCode::new(
            0x20,
            "JSR",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xA9,
            "LDA",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xA5,
            "LDA",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xB5,
            "LDA",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xAD,
            "LDA",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xBD,
            "LDA",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xB9,
            "LDA",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xA1,
            "LDA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xB1,
            "LDA",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0xA2,
            "LDX",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xA6,
            "LDX",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xB6,
            "LDX",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_Y
        ),
        OpCode::new(
            0xAE,
            "LDX",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xBE,
            "LDX",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xA0,
            "LDY",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xA4,
            "LDY",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xB4,
            "LDY",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xAC,
            "LDY",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xBC,
            "LDY",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x4A,
            "LSR",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Accumulator
        ),
        OpCode::new(
            0x46,
            "LSR",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x56,
            "LSR",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x4E,
            "LSR",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x5E,
            "LSR",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xEA,
            "NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x09,
            "ORA",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x05,
            "ORA",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x15,
            "ORA",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x0D,
            "ORA",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x1D,
            "ORA",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x19,
            "ORA",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x01,
            "ORA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x11,
            "ORA",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x48,
            "PHA",
            1,
            3,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x08,
            "PHP",
            1,
            3,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x68,
            "PLA",
            1,
            4,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x28,
            "PLP",
            1,
            4,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x2A,
            "ROL",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Accumulator
        ),
        OpCode::new(
            0x26,
            "ROL",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x36,
            "ROL",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x2E,
            "ROL",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x3E,
            "ROL",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x6A,
            "ROR",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Accumulator
        ),
        OpCode::new(
            0x66,
            "ROR",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x76,
            "ROR",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x6E,
            "ROR",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x7E,
            "ROR",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x40,
            "RTI",
            1,
            6,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x60,
            "RTS",
            1,
            6,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xE9,
            "SBC",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xE5,
            "SBC",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xF5,
            "SBC",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xED,
            "SBC",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xFD,
            "SBC",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xF9,
            "SBC",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xE1,
            "SBC",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xF1,
            "SBC",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x38,
            "SEC",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xF8,
            "SED",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x78,
            "SEI",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x85,
            "STA",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x95,
            "STA",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x8D,
            "STA",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x9D,
            "STA",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x99,
            "STA",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x81,
            "STA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x91,
            "STA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x86,
            "STX",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x96,
            "STX",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_Y
        ),
        OpCode::new(
            0x8E,
            "STX",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x84,
            "STY",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x94,
            "STY",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x8C,
            "STY",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xAA,
            "TAX",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xA8,
            "TAY",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xBA,
            "TSX",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x8A,
            "TXA",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x9A,
            "TXS",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x98,
            "TYA",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x0B,
            "*ANC",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x2B,
            "*ANC",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x87,
            "*SAX",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x97,
            "*SAX",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_Y
        ),
        OpCode::new(
            0x83,
            "*SAX",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x8F,
            "*SAX",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x6B,
            "*ARR",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x4B,
            "*ASR",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xAB,
            "*LXA",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x9F,
            "*SHA",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x93,
            "*SHA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0xCB,
            "*SBX",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xC7,
            "*DCP",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xD7,
            "*DCP",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xCF,
            "*DCP",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xDF,
            "*DCP",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xDB,
            "*DCP",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xC3,
            "*DCP",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xD3,
            "*DCP",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x04,
            "*NOP",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x14,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x34,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x44,
            "*NOP",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x54,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x64,
            "*NOP",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x74,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x80,
            "*NOP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x82,
            "*NOP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x89,
            "*NOP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xC2,
            "*NOP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xD4,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xE2,
            "*NOP",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0xF4,
            "*NOP",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xE7,
            "*ISB",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xF7,
            "*ISB",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0xEF,
            "*ISB",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xFF,
            "*ISB",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xFB,
            "*ISB",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xE3,
            "*ISB",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xF3,
            "*ISB",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x02,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x12,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x22,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x32,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x42,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x52,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x62,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x72,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x92,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xB2,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xD2,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xF2,
            "*JAM",
            1,
            0,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xBB,
            "*LAE",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xA7,
            "*LAX",
            2,
            3,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0xB7,
            "*LAX",
            2,
            4,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_Y
        ),
        OpCode::new(
            0xAF,
            "*LAX",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0xBF,
            "*LAX",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0xA3,
            "*LAX",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0xB3,
            "*LAX",
            2,
            5,
            CycleCalcMode::Page,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x1A,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x3A,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x5A,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x7A,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xDA,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0xFA,
            "*NOP",
            1,
            2,
            CycleCalcMode::None,
            AddressingMode::Implied
        ),
        OpCode::new(
            0x27,
            "*RLA",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x37,
            "*RLA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x2F,
            "*RLA",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x3F,
            "*RLA",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x3B,
            "*RLA",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x23,
            "*RLA",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x33,
            "*RLA",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x67,
            "*RRA",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x77,
            "*RRA",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x6F,
            "*RRA",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x7F,
            "*RRA",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x7B,
            "*RRA",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x63,
            "*RRA",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x73,
            "*RRA",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0xEB,
            "*SBC",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x07,
            "*SLO",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x17,
            "*SLO",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x0F,
            "*SLO",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x1F,
            "*SLO",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x1B,
            "*SLO",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x03,
            "*SLO",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x13,
            "*SLO",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x47,
            "*SRE",
            2,
            5,
            CycleCalcMode::None,
            AddressingMode::ZeroPage
        ),
        OpCode::new(
            0x57,
            "*SRE",
            2,
            6,
            CycleCalcMode::None,
            AddressingMode::ZeroPage_X
        ),
        OpCode::new(
            0x4F,
            "*SRE",
            3,
            6,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x5F,
            "*SRE",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x5B,
            "*SRE",
            3,
            7,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x43,
            "*SRE",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_X
        ),
        OpCode::new(
            0x53,
            "*SRE",
            2,
            8,
            CycleCalcMode::None,
            AddressingMode::Indirect_Y
        ),
        OpCode::new(
            0x9E,
            "*SHX",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
        OpCode::new(
            0x9C,
            "*SHY",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x0C,
            "*NOP",
            3,
            4,
            CycleCalcMode::None,
            AddressingMode::Absolute
        ),
        OpCode::new(
            0x1C,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x3C,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x5C,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x7C,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xDC,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0xFC,
            "*NOP",
            3,
            4,
            CycleCalcMode::Page,
            AddressingMode::Absolute_X
        ),
        OpCode::new(
            0x8B,
            "*ANE",
            2,
            2,
            CycleCalcMode::None,
            AddressingMode::Immediate
        ),
        OpCode::new(
            0x9B,
            "*SHS",
            3,
            5,
            CycleCalcMode::None,
            AddressingMode::Absolute_Y
        ),
    ];
}

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

pub trait NesCpuOps {
    fn adc(&mut self, mode: &AddressingMode);
    fn and(&mut self, mode: &AddressingMode);
    fn asl(&mut self, mode: &AddressingMode);
    fn bcc(&mut self, mode: &AddressingMode);
    fn bcs(&mut self, mode: &AddressingMode);
    fn beq(&mut self, mode: &AddressingMode);
    fn bit(&mut self, mode: &AddressingMode);
    fn bmi(&mut self, mode: &AddressingMode);
    fn bne(&mut self, mode: &AddressingMode);
    fn bpl(&mut self, mode: &AddressingMode);
    fn brk(&mut self, mode: &AddressingMode);
    fn bvc(&mut self, mode: &AddressingMode);
    fn bvs(&mut self, mode: &AddressingMode);
    fn clc(&mut self, mode: &AddressingMode);
    fn cld(&mut self, mode: &AddressingMode);
    fn cli(&mut self, mode: &AddressingMode);
    fn clv(&mut self, mode: &AddressingMode);
    fn cmp(&mut self, mode: &AddressingMode);
    fn cpx(&mut self, mode: &AddressingMode);
    fn cpy(&mut self, mode: &AddressingMode);
    fn dec(&mut self, mode: &AddressingMode);
    fn dex(&mut self, mode: &AddressingMode);
    fn dey(&mut self, mode: &AddressingMode);
    fn eor(&mut self, mode: &AddressingMode);
    fn inc(&mut self, mode: &AddressingMode);
    fn inx(&mut self, mode: &AddressingMode);
    fn iny(&mut self, mode: &AddressingMode);
    fn jmp(&mut self, mode: &AddressingMode);
    fn jsr(&mut self, mode: &AddressingMode);
    fn lda(&mut self, mode: &AddressingMode);
    fn ldx(&mut self, mode: &AddressingMode);
    fn ldy(&mut self, mode: &AddressingMode);
    fn lsr(&mut self, mode: &AddressingMode);
    fn nop(&mut self, mode: &AddressingMode);
    fn ora(&mut self, mode: &AddressingMode);
    fn pha(&mut self, mode: &AddressingMode);
    fn php(&mut self, mode: &AddressingMode);
    fn pla(&mut self, mode: &AddressingMode);
    fn plp(&mut self, mode: &AddressingMode);
    fn rol(&mut self, mode: &AddressingMode);
    fn ror(&mut self, mode: &AddressingMode);
    fn rti(&mut self, mode: &AddressingMode);
    fn rts(&mut self, mode: &AddressingMode);
    fn sbc(&mut self, mode: &AddressingMode);
    fn sec(&mut self, mode: &AddressingMode);
    fn sed(&mut self, mode: &AddressingMode);
    fn sei(&mut self, mode: &AddressingMode);
    fn sta(&mut self, mode: &AddressingMode);
    fn stx(&mut self, mode: &AddressingMode);
    fn sty(&mut self, mode: &AddressingMode);
    fn tax(&mut self, mode: &AddressingMode);
    fn tay(&mut self, mode: &AddressingMode);
    fn tsx(&mut self, mode: &AddressingMode);
    fn txa(&mut self, mode: &AddressingMode);
    fn txs(&mut self, mode: &AddressingMode);
    fn tya(&mut self, mode: &AddressingMode);
    fn anc(&mut self, mode: &AddressingMode);
    fn sax(&mut self, mode: &AddressingMode);
    fn arr(&mut self, mode: &AddressingMode);
    fn asr(&mut self, mode: &AddressingMode);
    fn lxa(&mut self, mode: &AddressingMode);
    fn sha(&mut self, mode: &AddressingMode);
    fn sbx(&mut self, mode: &AddressingMode);
    fn dcp(&mut self, mode: &AddressingMode);
    fn isb(&mut self, mode: &AddressingMode);
    fn jam(&mut self, mode: &AddressingMode);
    fn lae(&mut self, mode: &AddressingMode);
    fn lax(&mut self, mode: &AddressingMode);
    fn rla(&mut self, mode: &AddressingMode);
    fn rra(&mut self, mode: &AddressingMode);
    fn slo(&mut self, mode: &AddressingMode);
    fn sre(&mut self, mode: &AddressingMode);
    fn shx(&mut self, mode: &AddressingMode);
    fn shy(&mut self, mode: &AddressingMode);
    fn ane(&mut self, mode: &AddressingMode);
    fn shs(&mut self, mode: &AddressingMode);
}
