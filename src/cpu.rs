use crate::mem::*;
use std::convert::TryInto;
use std::convert::From;
use std::rc::Rc;
// use std::num::Wrapping;

pub const NEGATIVE_FLG: u8 = 0b1000_0000;           // bit7: N Flag. ネガティブフラグ。演算の結果が負の場合にセットされる。
pub const OVERFLOW_FLG: u8 = 0b0100_0000;           // bit6: V Flag. オーバーフローフラグ。符号付き演算の結果がオーバーフローした場合にセットされる。
pub const R_FLG: u8 = 0b0010_0000;                  // bit5: R Flag. Reaerved.予約済 (常に1固定)
pub const BREAK_COMMAND_FLG: u8 = 0b0001_0000;      // bit4: B Flag. ブレークコマンドフラグ。BRK命令が実行されたときにセットされる。
pub const DECIMAL_MODE_FLG: u8 = 0b0000_1000;       // bit3: D Flag. 10進モードフラグ。BCD（Binary-Coded Decimal）演算のためのアドレッシングモードを制御する。
pub const INTERRUPT_DISABLE_FLG: u8 = 0b0000_0100;  // bit2: I Flag. 割り込み無効フラグ (0 ... IRQ許可, 1 ... IRQをマスク)
pub const ZERO_FLG: u8 = 0b0000_0010;               // bit1: Z Flag. ゼロフラグ。演算の結果がゼロの場合にセットされる。
pub const CARRY_FLG: u8 = 0b0000_0001;              // bit0: C Flag. キャリーフラグ。算術演算でのキャリーや借りがある場合にセットされる。

enum CPUReg {
    A,   // 汎用レジスタ（アキュムレータ）... 演算の結果やデータを一時的に保持する。
    X,   // インデックスレジスタX         ... ループや配列のインデックスなどに使用する。
    Y,   // インデックスレジスタY         ... ループや配列のインデックスなどに使用する。
    SP,  // スタックポインタ              ... スタックのトップアドレスを示す。
}

struct ProgramCounter {
    pc: u16,
}

impl ProgramCounter {
    fn new() -> Self {
        ProgramCounter {
            // TODO PCの初期位置
            pc : ADDR_PRG_ROM,

             // リセットベクタ
            // pc : Self::ADDR_VEC_TBL_RST,
        }
    }
}

enum OpcodeType {
    // Load/Store Operations
    LDA, LDX, LDY, STA, STX, STY,
    // Register Transfer Operations
    TAX, TAY, TXA, TYA,
    // Stack Operations
    TSX, TXS, PHA, PHP, PLA, PLP,
    // Logical Operations
    AND, ORA, EOR, BIT,
    // Arithmetic Operations
    ADC, SBC, CMP, CPX, CPY, INC, INX, INY, DEC, DEX, DEY,
    // Shift and Rotate Operations
    ASL, LSR, ROL, ROR,
    // Jump and Call Operations
    JMP, JSR,
    // Branch Operations
    BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS,
    // Status Flag Operations
    CLC, CLD, CLI, CLV, SEC, SED, SEI,
    // Interrupt Operations
    RTS, RTI, BRK,
    // Other
    NOP, STP,
    // Undefined OP
    UNK,
}

enum AddrMode {
    ACC,IMM,
    ZPG,ZpgX,ZpgY,
    ABS,AbsX,AbsY,
    IND,IndX,IndY,
    REL,IMPL,
}

struct Opcode {
    opcode_type: OpcodeType,
}

#[derive(Clone)]
struct Addressing {
    addr_mode: Rc<AddrMode>,
}

trait CPU<T> {
    fn reset(&mut self);
    fn read(&mut self, address: u16) -> T;
    fn write(&mut self, address: u16, data: T);
    fn get_register(&self, register: CPUReg) -> T;
    fn set_register(&mut self, register: CPUReg, value: T);
    fn fetch_instruction(&mut self) -> T;
    fn read_operand(&mut self, addressing: Addressing) -> (Option<T>, Option<T>);
    fn decode_instruction(&mut self, op_code: T) -> (Opcode, Addressing);
    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing);
    fn push_stack(&mut self, data: T);
    fn pop_stack(&mut self) -> T;
}

/// RP2A03のステータスレジスタ
struct StatusRegister {
    p_reg: u8,
}

impl StatusRegister {
    fn new() -> Self {
        StatusRegister {
            p_reg: R_FLG, // ビット5: Reaerved.予約済 (常に1固定)
        }
    }

    fn cls_status_flg(&mut self, flg: u8) {
        self.p_reg &= !flg;
    }

    fn set_status_flg(&mut self, flg: u8) {
        self.p_reg |= flg;
    }

    fn get_status_flg(&self, flg: u8) -> bool {
        (self.p_reg & flg) != 0
    }

    fn get_status_flg_all(&self) -> u8 {
        self.p_reg
    }

    fn set_status_flg_all(&mut self, val: u8) {
        self.p_reg = val;
    }

    // fn cls_status_flg_all(&mut self) {
    //     self.p_reg = R_FLG;
    // }

    fn nzv_flg_update(&mut self, val: u8) {
        if val == 0{
            self.set_status_flg(ZERO_FLG);
        }else{
            self.cls_status_flg(ZERO_FLG);
        }

        if (val & BIN_BIT_7) != 0 {
            self.set_status_flg(NEGATIVE_FLG);
        }else{
            self.cls_status_flg(NEGATIVE_FLG);
        }
    }

    fn c_flg_update_add(&mut self, val_a: u8,  val_b: u8) -> u8{
        let mut ret: u16 = val_a as u16;
        ret += val_b as u16;
        if ret >  0x00FF {
            self.set_status_flg(CARRY_FLG);
            0x00
        }else{
            self.cls_status_flg(CARRY_FLG);
            ret as u8
        }
    }

    fn c_flg_update_l_shit(&mut self, val: u8) -> u8{
        let mut ret: u16 = val as u16;

        if (val & BIN_BIT_7) != 0 {
            self.set_status_flg(CARRY_FLG);
        }else {
            self.cls_status_flg(CARRY_FLG);
        }

        ret = ret << 1;
        if ret >  0x00FF {
            ret = ret & 0x00FF;
        }
        ret as u8
    }

    fn c_flg_update_r_shit(&mut self, val: u8) -> u8{
        let mut ret: i16 = val as i16;

        if (val & BIN_BIT_0) != 0 {
            self.set_status_flg(CARRY_FLG);
        }else {
            self.cls_status_flg(CARRY_FLG);
        }

        ret = ret >> 1;
        if ret <= 0x00 {
            ret = 0;
        }
        ret as u8
    }
}

struct RP2A03<T> {
    cpu_reg: [T; 4],
    cpu_p_reg: StatusRegister,
    cpu_pc: ProgramCounter,
    nes_mem: NESMemory
}

impl<T> CPU<T> for RP2A03<T>
where
    T: Copy + From<u8> + std::ops::Add<Output = T> + std::ops::Sub<Output = T>
        + std::ops::BitAnd<Output = T> + std::ops::BitOr<Output = T>+ std::ops::BitXor<Output = T>
        + TryFrom<u16> + Into<u8> + Into<u16> + Into<u32> + Into<i16> + Into<i32>
        + PartialEq + PartialOrd + std::ops::Shl<u8, Output = T>
        + std::ops::Shr<Output = T> + std::ops::Shl<Output = T> + std::ops::BitOrAssign,
    <T as std::convert::TryFrom<u16>>::Error: std::fmt::Debug,i32: From<T>,
{
    fn reset(&mut self){
        self.set_register(CPUReg::A, T::from(0u8));
        self.set_register(CPUReg::X, T::from(0u8));
        self.set_register(CPUReg::Y, T::from(0u8));
        self.set_register(CPUReg::SP, T::from(0xFFu8));
    }

    fn read(&mut self, address: u16) -> T
    where T: From<u8>,
    {
        T::from(self.nes_mem.mem_read(address))
    }

    fn write(&mut self, address: u16, data: T)
    where T: Into<u8>,
    {
        self.nes_mem.mem_write(address, data.into());
    }

    fn get_register(&self, register: CPUReg) -> T {
        match register {
            CPUReg::A => self.cpu_reg[0],
            CPUReg::X => self.cpu_reg[1],
            CPUReg::Y => self.cpu_reg[2],
            CPUReg::SP => self.cpu_reg[3],
        }
    }

    fn set_register(&mut self, register: CPUReg, value: T) {
        match register {
            CPUReg::A => self.cpu_reg[0] = value,
            CPUReg::X => self.cpu_reg[1] = value,
            CPUReg::Y => self.cpu_reg[2] = value,
            CPUReg::SP => self.cpu_reg[3] = value,
        }
    }

    fn fetch_instruction(&mut self) -> T {
        let op_code = self.read(self.cpu_pc.pc);
        op_code
    }

    fn decode_instruction(&mut self, op_code: T) -> (Opcode, Addressing) {
        let opcode_type: OpcodeType;
        let addr_mode: Rc<AddrMode>;

        match op_code.into() {
            0x00 => { opcode_type = OpcodeType::BRK; addr_mode = Rc::new(AddrMode::IMPL); },
            0x01 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::IndX); },
            0x05 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::ZPG); },
            0x06 => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ZPG); },
            0x08 => { opcode_type = OpcodeType::PHP; addr_mode = Rc::new(AddrMode::IMPL); },
            0x09 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::IMM); },
            0x0A => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ACC); },
            0x0D => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::ABS); },
            0x0E => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ABS); },
            0x10 => { opcode_type = OpcodeType::BPL; addr_mode = Rc::new(AddrMode::REL); },
            0x11 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::IndY); },
            0x15 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x16 => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x18 => { opcode_type = OpcodeType::CLC; addr_mode = Rc::new(AddrMode::IMPL); },
            0x19 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::AbsY); },
            0x1D => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::AbsX); },
            0x1E => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::AbsX); },
            0x20 => { opcode_type = OpcodeType::JSR; addr_mode = Rc::new(AddrMode::ABS); },
            0x21 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::IndX); },
            0x24 => { opcode_type = OpcodeType::BIT; addr_mode = Rc::new(AddrMode::ZPG); },
            0x25 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::ZPG); },
            0x26 => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ZPG); },
            0x28 => { opcode_type = OpcodeType::PLP; addr_mode = Rc::new(AddrMode::IMPL); },
            0x29 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::IMM); },
            0x2A => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ACC); },
            0x2C => { opcode_type = OpcodeType::BIT; addr_mode = Rc::new(AddrMode::ABS); },
            0x2D => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::ABS); },
            0x2E => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ABS); },
            0x30 => { opcode_type = OpcodeType::BMI; addr_mode = Rc::new(AddrMode::REL); },
            0x31 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::IndY); },
            0x35 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x36 => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x38 => { opcode_type = OpcodeType::SEC; addr_mode = Rc::new(AddrMode::IMPL); },
            0x39 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::AbsY); },
            0x3D => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::AbsX); },
            0x3E => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::AbsX); },
            0x40 => { opcode_type = OpcodeType::RTI; addr_mode = Rc::new(AddrMode::IMPL); },
            0x41 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::IndX); },
            0x45 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::ZPG); },
            0x46 => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ZPG); },
            0x48 => { opcode_type = OpcodeType::PHA; addr_mode = Rc::new(AddrMode::IMPL); },
            0x49 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::IMM); },
            0x4A => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ACC); },
            0x4C => { opcode_type = OpcodeType::JMP; addr_mode = Rc::new(AddrMode::ABS); },
            0x4D => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::ABS); },
            0x4E => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ABS); },
            0x50 => { opcode_type = OpcodeType::BVC; addr_mode = Rc::new(AddrMode::REL); },
            0x51 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::IndY); },
            0x55 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x56 => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x58 => { opcode_type = OpcodeType::CLI; addr_mode = Rc::new(AddrMode::IMPL); },
            0x59 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::AbsY); },
            0x5D => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::AbsX); },
            0x5E => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::AbsX); },
            0x60 => { opcode_type = OpcodeType::RTS; addr_mode = Rc::new(AddrMode::IMPL); },
            0x61 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::IndX); },
            0x65 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::ZPG); },
            0x66 => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ZPG); },
            0x68 => { opcode_type = OpcodeType::PLA; addr_mode = Rc::new(AddrMode::IMPL); },
            0x69 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::IMM); },
            0x6A => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ACC); },
            0x6C => { opcode_type = OpcodeType::JMP; addr_mode = Rc::new(AddrMode::IND); },
            0x6D => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::ABS); },
            0x6E => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ABS); },
            0x70 => { opcode_type = OpcodeType::BVS; addr_mode = Rc::new(AddrMode::REL); },
            0x71 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::IndY); },
            0x75 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x76 => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x78 => { opcode_type = OpcodeType::SEI; addr_mode = Rc::new(AddrMode::IMPL); },
            0x79 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::AbsY); },
            0x7D => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::AbsX); },
            0x7E => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::AbsX); },
            0x81 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::IndX); },
            0x84 => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::ZPG); },
            0x85 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::ZPG); },
            0x86 => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::ZPG); },
            0x88 => { opcode_type = OpcodeType::DEY; addr_mode = Rc::new(AddrMode::IMPL); },
            0x8A => { opcode_type = OpcodeType::TXA; addr_mode = Rc::new(AddrMode::IMPL); },
            0x8C => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::ABS); },
            0x8D => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::ABS); },
            0x8E => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::ABS); },
            0x90 => { opcode_type = OpcodeType::BCC; addr_mode = Rc::new(AddrMode::REL); },
            0x91 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::IndY); },
            0x94 => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x95 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x96 => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::ZpgY); },
            0x98 => { opcode_type = OpcodeType::TYA; addr_mode = Rc::new(AddrMode::IMPL); },
            0x99 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::AbsY); },
            0x9A => { opcode_type = OpcodeType::TXS; addr_mode = Rc::new(AddrMode::IMPL); },
            0x9D => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::AbsX); },
            0xA0 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::IMM); },
            0xA1 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::IndX); },
            0xA2 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::IMM); },
            0xA4 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::ZPG); },
            0xA5 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::ZPG); },
            0xA6 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::ZPG); },
            0xA8 => { opcode_type = OpcodeType::TAY; addr_mode = Rc::new(AddrMode::IMPL); },
            0xA9 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::IMM); },
            0xAA => { opcode_type = OpcodeType::TAX; addr_mode = Rc::new(AddrMode::IMPL); },
            0xAC => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::ABS); },
            0xAD => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::ABS); },
            0xAE => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::ABS); },
            0xB0 => { opcode_type = OpcodeType::BCS; addr_mode = Rc::new(AddrMode::REL); },
            0xB1 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::IndY); },
            0xB4 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xB5 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xB6 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::ZpgY); },
            0xB8 => { opcode_type = OpcodeType::CLV; addr_mode = Rc::new(AddrMode::IMPL); },
            0xB9 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::AbsY); },
            0xBA => { opcode_type = OpcodeType::TSX; addr_mode = Rc::new(AddrMode::IMPL); },
            0xBC => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::AbsX); },
            0xBD => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::AbsX); },
            0xBE => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::AbsY); },
            0xC0 => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::IMM); },
            0xC1 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::IndX); },
            0xC4 => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::ZPG); },
            0xC5 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::ZPG); },
            0xC6 => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::ZPG); },
            0xC8 => { opcode_type = OpcodeType::INY; addr_mode = Rc::new(AddrMode::IMPL); },
            0xC9 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::IMM); },
            0xCA => { opcode_type = OpcodeType::DEX; addr_mode = Rc::new(AddrMode::IMPL); },
            0xCC => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::ABS); },
            0xCD => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::ABS); },
            0xCE => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::ABS); },
            0xD0 => { opcode_type = OpcodeType::BNE; addr_mode = Rc::new(AddrMode::REL); },
            0xD1 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::IndY); },
            0xD5 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xD6 => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xD8 => { opcode_type = OpcodeType::CLD; addr_mode = Rc::new(AddrMode::IMPL); },
            0xD9 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::AbsY); },
            0xDD => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::AbsX); },
            0xDE => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::AbsX); },
            0xE0 => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::IMM); },
            0xE1 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::IndX); },
            0xE4 => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::ZPG); },
            0xE5 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::ZPG); },
            0xE6 => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::ZPG); },
            0xE8 => { opcode_type = OpcodeType::INX; addr_mode = Rc::new(AddrMode::IMPL); },
            0xE9 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::IMM); },
            0xEC => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::ABS); },
            0xED => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::ABS); },
            0xEE => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::ABS); },
            0xF0 => { opcode_type = OpcodeType::BEQ; addr_mode = Rc::new(AddrMode::REL); },
            0xF1 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::IndY); },
            0xF5 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xF6 => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::ZpgX); },
            0xF8 => { opcode_type = OpcodeType::SED; addr_mode = Rc::new(AddrMode::IMPL); },
            0xF9 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::AbsY); },
            0xFD => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::AbsX); },
            0xFE => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::AbsX); },

            // NOP
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::IMPL); },
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::IMM); },
            0x04 | 0x44 | 0x64 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::ZPG); },
            0x14 | 0x34 | 0x54 | 0x74| 0xD4| 0xF4 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::ZpgX); },
            0x0C => { opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::ABS); },
            0x1C | 0x3C | 0x5C | 0x7C| 0xDC| 0xFC => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::AbsX); },

            // STP
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2  => {
                opcode_type = OpcodeType::STP; addr_mode = Rc::new(AddrMode::IMPL); },

            _ => { opcode_type = OpcodeType::UNK; addr_mode = Rc::new(AddrMode::IMPL); }
        };

        let opcode: Opcode = Opcode { opcode_type };
        let addressing: Addressing = Addressing { addr_mode };

        (opcode, addressing)
    }

    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing) {
        let (operand,operand_second) = self.read_operand(addressing);

        match opcode.opcode_type {
            OpcodeType::NOP => {
                // No operation, do nothing
                println!("NOP");
            }

            // // Logical Operations / 論理演算命令
            OpcodeType::AND => {
                let a: T = self.get_register(CPUReg::A);
                if let Some(operand_value) = operand {
                    let result: T = a & operand_value;
                    self.set_register(CPUReg::A, result);
                }
                println!("AND");
            }
            OpcodeType::ORA => {
                let a: T = self.get_register(CPUReg::A);
                if let Some(operand_value) = operand {
                    let result: T = a | operand_value;
                    self.set_register(CPUReg::A, result);
                }
                println!("ORA");
            }
            OpcodeType::EOR => {
                let a: T = self.get_register(CPUReg::A);
                if let Some(operand_value) = operand {
                    let result: T = a ^ operand_value;
                    self.set_register(CPUReg::A, result);
                }
                println!("EOR");
            }
            OpcodeType::BIT => {
                let a: T = self.get_register(CPUReg::A);
                if let Some(operand_value) = operand {
                    let result: T = a & operand_value;
                    if result == T::from(0) {
                        self.cpu_p_reg.set_status_flg(ZERO_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(ZERO_FLG);
                    }
                    if (operand_value & T::from(BIN_BIT_7)) != T::from(0) {
                        self.cpu_p_reg.set_status_flg(NEGATIVE_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(NEGATIVE_FLG);
                    }
                    if (operand_value & T::from(BIN_BIT_6)) != T::from(0) {
                        self.cpu_p_reg.set_status_flg(OVERFLOW_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(OVERFLOW_FLG);
                    }
                }
                println!("BIT");
            }

            // Arithmetic Operations / 算術倫理演算
            OpcodeType::ADC => {
                if let Some(value) = operand {
                    let val: T = value.into();
                    let a: T = self.get_register(CPUReg::A);
                    let mut carry = T::from(0x00);
                    if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                        carry = T::from(0x01);
                    }
                    let result: T = a + carry;
                    let ret: u8 = self.cpu_p_reg.c_flg_update_add(result.try_into().unwrap(), val.try_into().unwrap());
                    self.set_register(CPUReg::A, ret.try_into().unwrap());
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                }
                println!("ADC");
            }
            OpcodeType::SBC => {
                if let Some(value) = operand {
                    let val: T = value.into();
                    let a = self.get_register(CPUReg::A);
                    let mut carry: T = T::from(0x00);
                    let mut ret = T::from(0x00);
                    if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                        carry = T::from(0x01);
                        // A = A-M-(1-C)
                        let result: T = a - val;
                        ret = result;
                    }else{
                        let result: T = a - val - carry;
                        ret = result;
                    }
                    self.set_register(CPUReg::A, ret);
                    self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                }
                println!("SBC");
            }
            OpcodeType::CMP => {
                if let Some(operand_value) = operand {
                    let a = self.get_register(CPUReg::A);
                    let result: T = a - operand_value;
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                }
                println!("CMP");
            }
            OpcodeType::CPX => {
                if let Some(operand_value) = operand {
                    let x: T = self.get_register(CPUReg::X);
                    let result: T = x - operand_value;
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                }
                println!("CPX");
            }
            OpcodeType::CPY => {
                if let Some(operand_value) = operand {
                    let y: T = self.get_register(CPUReg::X);
                    let result: T = y - operand_value;
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                }
                println!("CPY");
            }
            OpcodeType::INC => {
                if let Some(operand_value) = operand {
                    let ret: u8 = self.cpu_p_reg.c_flg_update_add(operand_value.try_into().unwrap(), 1);
                    self.set_register(CPUReg::A, ret.try_into().unwrap());
                    self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                }
                println!("INC");
            }
            OpcodeType::INX => {
                let x: T = self.get_register(CPUReg::X);
                let ret: u8 = self.cpu_p_reg.c_flg_update_add(x.try_into().unwrap(), 1);
                self.set_register(CPUReg::X, ret.try_into().unwrap());
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                println!("INX");
            }
            OpcodeType::INY => {
                let y: T = self.get_register(CPUReg::Y);
                let ret: u8 = self.cpu_p_reg.c_flg_update_add(y.try_into().unwrap(), 1);
                self.set_register(CPUReg::X, ret.try_into().unwrap());
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                println!("INY");
            }
            OpcodeType::DEC => {
                if let Some(operand_value) = operand {
                    let result: T = operand_value - T::from(0x01);
                    self.set_register(CPUReg::A, result);
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                }
                println!("DEC");
            }
            OpcodeType::DEX => {
                let x: T = self.get_register(CPUReg::X);
                let result: T = x - T::from(0x01);
                self.set_register(CPUReg::X, result);
                self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                println!("DEX");
            }
            OpcodeType::DEY => {
                let y: T = self.get_register(CPUReg::Y);
                let result: T = y - T::from(0x01);
                self.set_register(CPUReg::Y, result);
                self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
                println!("DEY");
            }

            // Shift and Rotate Operations
            OpcodeType::ASL => {
                let a: T = self.get_register(CPUReg::A);
                let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(a.try_into().unwrap());
                ret = ret & 0xFE; // bit0, clear
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                self.set_register(CPUReg::A, ret.try_into().unwrap());
                println!("ASL");
            }
            OpcodeType::LSR => {
                let a: T = self.get_register(CPUReg::A);
                let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(a.try_into().unwrap());
                ret = ret & 0x7F; // bit7, clear
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                self.set_register(CPUReg::A, ret.try_into().unwrap());
                println!("LSR");
            }
            OpcodeType::ROL => {
                let a: T = self.get_register(CPUReg::A);
                let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(a.try_into().unwrap());
                let mut carry: u8 = 0;
                if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                    carry = BIN_BIT_0;
                }
                ret = ret | carry; // bit0 = C Flag Set
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                self.set_register(CPUReg::A, ret.try_into().unwrap());
                println!("ROL");
            }
            OpcodeType::ROR => {
                let a: T = self.get_register(CPUReg::A);
                let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(a.try_into().unwrap());
                let mut carry: u8 = 0;
                if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                    carry = BIN_BIT_7;
                }
                ret = ret | carry; // bit7 = C Flag Set
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                self.set_register(CPUReg::A, ret.try_into().unwrap());
                println!("ROR");
            }

            // Load/Store Operations
            OpcodeType::LDA => {
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    ret = val;
                    println!("LDA {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2.try_into().unwrap();
                        println!("LDA {:#02X} {:#02X}", val, val2);
                    }
                }
                self.set_register(CPUReg::A, ret.try_into().unwrap());
            }
            OpcodeType::LDX => {
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    ret = val;
                    println!("LDX {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2.try_into().unwrap();
                        println!("LDX {:#02X} {:#02X}", val, val2);
                    }
                }
                self.set_register(CPUReg::X, ret.try_into().unwrap());
            }
            OpcodeType::LDY => {
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    ret = val;
                    println!("LDY {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2.try_into().unwrap();
                        println!("LDY {:#02X} {:#02X}", val, val2);
                    }
                }
                self.set_register(CPUReg::Y, ret.try_into().unwrap());
            }
            OpcodeType::STA => {
                let a: T = self.get_register(CPUReg::A);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = self.cpu_pc.pc | val as u16;
                    println!("STA {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                        println!("STA {:#02X} {:#02X}", val, val2);
                    }
                }
                self.write(addr, a);
            }
            OpcodeType::STX => {
                let x: T = self.get_register(CPUReg::X);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = self.cpu_pc.pc | val as u16;
                    println!("STX {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                        println!("STX {:#02X} {:#02X}", val, val2);
                    }
                }
                self.write(addr, x);
            }
            OpcodeType::STY => {
                let y: T = self.get_register(CPUReg::Y);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = self.cpu_pc.pc | val as u16;
                    println!("STY {:#02X}", val);

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                        println!("STY {:#02X} {:#02X}", val, val2);
                    }
                }
                self.write(addr, y);
            }

            // Register Transfer Operations/レジスタ転送関連の命令
            OpcodeType::TAX => {
                let a = self.get_register(CPUReg::A);
                self.set_register(CPUReg::X, a);
                println!("TAX");
            }
            OpcodeType::TAY => {
                let a = self.get_register(CPUReg::A);
                self.set_register(CPUReg::Y, a);
                println!("TAY");
            }
            OpcodeType::TXA => {
                let x = self.get_register(CPUReg::X);
                self.set_register(CPUReg::A, x);
                println!("TXA");
            }
            OpcodeType::TYA => {
                let y = self.get_register(CPUReg::Y);
                self.set_register(CPUReg::A, y);
                println!("TYA");
            }

            // Stack Operations / スタック関連の命令
            OpcodeType::TSX => {
                let sp = self.get_register(CPUReg::SP);
                self.set_register(CPUReg::X, sp);
                println!("TSX");
            }
            OpcodeType::TXS => {
                let x = self.get_register(CPUReg::X);
                self.set_register(CPUReg::SP, x);
                println!("TXS");
            }
            OpcodeType::PHA => {
                let a = self.get_register(CPUReg::A);
                self.push_stack(a);
                println!("PHA");
            }
            OpcodeType::PHP => {
                let p = self.cpu_p_reg.get_status_flg_all();
                self.push_stack(p.try_into().unwrap());
                println!("PHP");
            }
            OpcodeType::PLA => {
                let value = self.pop_stack();
                self.set_register(CPUReg::A, value);
                self.cpu_p_reg.nzv_flg_update(value.try_into().unwrap());
                println!("PLA");
            }
            OpcodeType::PLP => {
                let value = self.pop_stack();
                self.cpu_p_reg.set_status_flg_all(value.try_into().unwrap());
                println!("PLP");
            }

            // Status Flag Operations / ステータスフラグ関連の命令
            OpcodeType::CLC => {
                self.cpu_p_reg.cls_status_flg(CARRY_FLG);
                println!("CLC");
            }
            OpcodeType::CLD => {
                self.cpu_p_reg.cls_status_flg(DECIMAL_MODE_FLG);
                println!("CLD");
            }
            OpcodeType::CLI => {
                self.cpu_p_reg.cls_status_flg(INTERRUPT_DISABLE_FLG);
                println!("CLI");
            }
            OpcodeType::CLV => {
                self.cpu_p_reg.cls_status_flg(OVERFLOW_FLG);
                println!("CLV");
            }
            OpcodeType::SEC => {
                self.cpu_p_reg.set_status_flg(CARRY_FLG);
                println!("SEC");
            }
            OpcodeType::SED => {
                self.cpu_p_reg.set_status_flg(DECIMAL_MODE_FLG);
                println!("SED");
            }
            OpcodeType::SEI => {
                self.cpu_p_reg.set_status_flg(INTERRUPT_DISABLE_FLG);
                println!("SEI");
            }

            // Jump and Call Operations
            OpcodeType::JMP => {
                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        let jump_addr: u16 = (val2 as u16) << 8 | val as u16;
                        self.cpu_pc.pc = jump_addr;
                        println!("JMP ${:04X}",jump_addr);
                    }
                }
            }
            OpcodeType::JSR => {
                self.cpu_pc.pc += 1;
                let return_addr: u16 = self.cpu_pc.pc;
                self.push_stack((return_addr & 0x00FF).try_into().unwrap());
                self.push_stack(((return_addr & 0xFF00) >> 0x0008).try_into().unwrap());

                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        let jump_addr: u16 = (val2 as u16) << 8 | val as u16;
                        self.cpu_pc.pc = jump_addr;
                        println!("JSR ${:04X}",jump_addr);
                    }
                }
            }

            // Branch Operations / 分岐命令
            OpcodeType::BCC => {
                let ret = self.cpu_p_reg.get_status_flg(CARRY_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BCC (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BCC Not Branch!");
            }
            OpcodeType::BCS => {
                let ret = self.cpu_p_reg.get_status_flg(CARRY_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BCS (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BCS Not Branch!");
            }
            OpcodeType::BEQ => {
                let ret = self.cpu_p_reg.get_status_flg(ZERO_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BEQ (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BEQ Not Branch!");
            }
            OpcodeType::BNE => {
                let ret = self.cpu_p_reg.get_status_flg(ZERO_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BNE (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BNE Not Branch!");
            }
            OpcodeType::BVC => {
                let ret = self.cpu_p_reg.get_status_flg(OVERFLOW_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BVC (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BVC Not Branch!");
            }
            OpcodeType::BVS => {
                let ret = self.cpu_p_reg.get_status_flg(OVERFLOW_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BVS (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BVS Not Branch!");
            }
            OpcodeType::BPL => {
                let ret = self.cpu_p_reg.get_status_flg(NEGATIVE_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BPK (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BPL Not Branch!");
            }
            OpcodeType::BMI => {
                let ret = self.cpu_p_reg.get_status_flg(NEGATIVE_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u8 = value.try_into().unwrap();
                            let val2: u8 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 as u16) << 8 | val as u16;
                            self.cpu_pc.pc = branch_addr;
                            println!("BMI (Branch ${:#04X})", branch_addr);
                        }
                    }
                }
                println!("BMI Not Branch!");
            }

            // Intrrupt Operations / 割込み関連
            OpcodeType::RTI => {
                println!("RTI");
                let status = self.pop_stack();
                self.cpu_p_reg.set_status_flg_all(status.into());
                let mut return_addr = self.pop_stack();
                return_addr |= self.pop_stack() << 8;
                self.cpu_pc.pc = return_addr.try_into().unwrap();
            }
            OpcodeType::RTS => {
                println!("RTS");
                let mut return_addr = self.pop_stack();
                return_addr |= self.pop_stack() << 8;
                self.cpu_pc.pc = return_addr.try_into().unwrap();
                self.cpu_pc.pc += 1;
            }
            OpcodeType::BRK => {
                if self.cpu_p_reg.get_status_flg(BREAK_COMMAND_FLG) != true {
                    print!("BRK(INT)");
                    self.cpu_pc.pc += 1;
                    self.cpu_p_reg.set_status_flg(BREAK_COMMAND_FLG);
                    self.push_stack((self.cpu_pc.pc & 0x00FF).try_into().unwrap());
                    self.push_stack(((self.cpu_pc.pc & 0xFF00) >> 0x0008).try_into().unwrap());
                    self.push_stack(self.cpu_p_reg.get_status_flg_all().try_into().unwrap());
                    self.cpu_p_reg.set_status_flg(BREAK_COMMAND_FLG);
                    let mut _jmp_addr: T = self.read(ADDR_VEC_TBL_IRQ);
                    _jmp_addr = self.read(ADDR_VEC_TBL_IRQ + 1) << 0x0008;
                    self.cpu_pc.pc = _jmp_addr.try_into().unwrap();
                    print!("Jmp to: ${:04X}", self.cpu_pc.pc);
                }
                println!("BRK(INT Mask)");
            }

            // Other
            OpcodeType::STP | _ => {
                // TODO STPと未定義命令をどうするか
                println!("Undefined Instruction!");
            }
        }

        // pc ++
        if operand != None
        {
            self.cpu_pc.pc += 1;
        }
    }

    fn push_stack(&mut self, data: T) {
        println!("Push Stack");
        let sp = self.get_register(CPUReg::SP);
        let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
        self.write(address, data);
        self.set_register(CPUReg::SP, sp - T::from(1u8));
    }

    fn pop_stack(&mut self) -> T {
        println!("POP Stack");
        let sp = self.get_register(CPUReg::SP);
        self.set_register(CPUReg::SP, sp + T::from(1u8));
        let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
        self.read(address)
    }

    fn read_operand(&mut self, addressing: Addressing) -> (Option<T>, Option<T>)
    {
        self.cpu_pc.pc += 1;
        match *addressing.addr_mode {
            AddrMode::ACC => {
                print!("OP-Code:(ACC) ");
                (Some(self.get_register(CPUReg::A)), None)
            }
            AddrMode::IMM => {
                print!("OP-Code:(IMM) ");
                (Some(self.read(self.cpu_pc.pc)), None)
            }
            AddrMode::ZPG => {
                print!("OP-Code:(ZPG) ");
                (Some(self.read(self.cpu_pc.pc)), None)
            }
            AddrMode::ZpgX => {
                print!("OP-Code:(ZpgX) ");
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                (Some(self.read(address.try_into().unwrap())),None)
            }
            AddrMode::ZpgY => {
                print!("OP-Code:(ZpgY) ");
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                (Some(self.read(address.try_into().unwrap())),None)
            }
            AddrMode::ABS => {
                print!("OP-Code:(ABS) ");
                let address_l:u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                self.cpu_pc.pc += 1;
                let address_u:u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                (Some(address_l.try_into().unwrap()), Some(address_u.try_into().unwrap()))
            }
            AddrMode::AbsX => {
                print!("OP-Code:(AbsX) ");
                let mut address_l: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                address_l |= TryInto::<u16>::try_into(self.get_register(CPUReg::X)).unwrap();
                self.cpu_pc.pc += 1;
                let mut address_u: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                address_u |= address_l;
                (Some(address_l.try_into().unwrap()), Some(address_u.try_into().unwrap()))
            }
            AddrMode::AbsY => {
                print!("OP-Code:(AbsY) ");
                let mut address_l: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                address_l |= TryInto::<u16>::try_into(self.get_register(CPUReg::Y)).unwrap();
                self.cpu_pc.pc += 1;
                let mut address_u: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                address_u |= address_l;
                (Some(address_l.try_into().unwrap()), Some(address_u.try_into().unwrap()))
            }
            AddrMode::IND => {
                print!("OP-Code:(IND) ");
                let address: T = self.read(self.cpu_pc.pc);
                (Some(self.read(address.try_into().unwrap())),None)
            }
            AddrMode::IndX => {
                print!("OP-Code:(IndX) ");
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                let address: T = self.read(base_address.try_into().unwrap());
                (Some(self.read(address.try_into().unwrap())),None)
            }
            AddrMode::IndY => {
                print!("OP-Code:(IndY) ");
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                let address: T = self.read(base_address.try_into().unwrap());
                (Some(self.read(address.try_into().unwrap())),None)
            }
            AddrMode::REL => { // Relative Addressing(相対アドレッシング)
                let offset: i16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                print!("OP-Code:(REL (Offset:{}))", offset);
                let addr: u16 = self.cpu_pc.pc.wrapping_add((offset & 0xff) as i16 as u16).wrapping_add(2).try_into().unwrap();
                (Some(addr.try_into().unwrap()), Some((addr >> 8).try_into().unwrap()))
            }
            AddrMode::IMPL => { // Implied Addressing
                print!("OP-Code:(IMPL) ");
                // Not, Have Operand
                // self.cpu_pc.pc = self.cpu_pc.pc - 1;
                (None, None)
            }
        }
    }
}

fn cpu_reg_show(cpu :&RP2A03<u8>)
{
    let a: u8 = cpu.get_register(CPUReg::A);
    let x: u8 = cpu.get_register(CPUReg::X);
    let y: u8 = cpu.get_register(CPUReg::Y);
    let sp: u8 = cpu.get_register(CPUReg::SP);
    let p: u8 = cpu.cpu_p_reg.get_status_flg_all();
    let pc: u16 = cpu.cpu_pc.pc;
    println!("[DEBUG] A:0x{:02X},X:0x{:02X},Y:0x{:02X},S:0x{:02X},P:{:08b},PC:0x{:04X}",a,x,y,sp,p,pc);
}

fn cpu_proc(cpu :&mut RP2A03<u8>)
{
    // println!("[DEBUG] : Fetch!");
    let op_code = cpu.fetch_instruction();
    // println!("[DEBUG] : Decode!");
    let (opcode, addressing) = cpu.decode_instruction(op_code);
    // println!("[DEBUG] : Execute!");
    cpu.execute_instruction(opcode, addressing);
}

static mut S_CPU: Option<RP2A03<u8>> = None;
static mut S_CPU_STOP: bool = false;

pub fn cpu_stop(flg: bool)
{
    unsafe {
        if S_CPU_STOP != false
        {
            println!("[DEBUG] : CPU Stop");
        }
    }
}

pub fn cpu_reset() {
    unsafe {
        S_CPU = Some(RP2A03 {
            cpu_reg: [0u8; 4],
            cpu_p_reg: StatusRegister::new(),
            cpu_pc: ProgramCounter::new(),
            nes_mem: NESMemory::new(),
        });
    }

    unsafe {
        if let Some(ref mut cpu) = S_CPU {
            cpu.nes_mem.mem_reset();
            cpu.reset();

        // (DEBUG)
            cpu.cpu_p_reg.set_status_flg(OVERFLOW_FLG);
        }
    }
}

pub fn cpu_main()
{
    unsafe {
        if S_CPU_STOP != true
        {
            // println!("[DEBUG] : CPU Main Loop");
                if let Some(ref mut cpu) = S_CPU {
                    cpu_proc(cpu);
                    cpu_reg_show(cpu);
                }
            }
    }
}

// ====================================== TEST ======================================
#[cfg(test)]
mod cpu_test {

    #[test]
    fn cpu_test()
    {
        // TODO CPU Test
    }
}
// ==================================================================================