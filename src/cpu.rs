use crate::mem::*;
use std::pin::Pin;
// use std::sync::{Once, Mutex};
// use lazy_static::lazy_static;
use once_cell::sync::Lazy;

pub const NEGATIVE_FLG: u8 = 0b1000_0000;           // bit7: N Flag. ネガティブフラグ。演算の結果が負の場合にセットされる。
pub const OVERFLOW_FLG: u8 = 0b0100_0000;           // bit6: V Flag. オーバーフローフラグ。符号付き演算の結果がオーバーフローした場合にセットされる。
pub const R_FLG: u8 = 0b0010_0000;                  // bit5: R Flag. Reaerved.予約済 (常に1固定)
pub const BREAK_COMMAND_FLG: u8 = 0b0001_0000;      // bit4: B Flag. ブレークコマンドフラグ。BRK命令が実行されたときにセットされる。
pub const DECIMAL_MODE_FLG: u8 = 0b0000_1000;       // bit3: D Flag. 10進モードフラグ。BCD（Binary-Coded Decimal）演算のためのアドレッシングモードを制御する。
pub const INTERRUPT_DISABLE_FLG: u8 = 0b0000_0100;  // bit2: I Flag. 割り込み無効フラグ (0 ... IRQ許可, 1 ... IRQをマスク)
pub const ZERO_FLG: u8 = 0b0000_0010;               // bit1: Z Flag. ゼロフラグ。演算の結果がゼロの場合にセットされる。
pub const CARRY_FLG: u8 = 0b0000_0001;              // bit0: C Flag. キャリーフラグ。算術演算でのキャリーや借りがある場合にセットされる。

pub const ADDR_VEC_TBL_RST: u16 = 0xFFFC;  // RESET Vector Table
pub const ADDR_VEC_TBL_NMI: u16 = 0xFFFA;  // NMI Vector Table
pub const ADDR_VEC_TBL_IRQ: u16 = 0xFFFE;  // IRQ Vector Table

#[derive(Clone)]
enum InterruptType {
    RST,
    NMI,
    IRQ,
}

#[derive(Clone)]
pub enum OpCode {
    // Load/Store Operations
    LDA, LDX, LDY, STA, STX, STY,
    // Register Transfer Operations
    TAX, TAY, TXA, TYA,
    // Stack Operations
    TSX, TXS, PHA, PHP, PLA, PLP,
    // Logical Operations
    AND, ORA, EOR,
    // Arithmetic Operations
    ADC, SBC, CMP, CPX, CPY, INC, INX, INY, DEC, DEX, DEY,
    // Shift and Rotate Operations
    ASL, LSR, ROL, ROR,
    // Jump and Call Operations
    JMP, JSR, RTS,
    // Branch Operations
    BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS,
    // Status Flag Operations
    BIT, CLC, CLD, CLI, CLV, SEC, SED, SEI,
    // Interrupt Operations
    RTI, BRK,
    // Other
    NOP, STP,
    // Undefined OP
    UNK,
}

#[derive(Clone)]
pub enum Addressing {
    ACC,IMM,
    ZPG,ZpgX,ZpgY,
    ABS,AbsX,AbsY,
    IND,IndX,IndY,
    REL,IMPL,
}

#[derive(Clone)]
pub struct RP2A03
{
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_p: u8,
    pub reg_sp: u8,
    pub reg_pc: u16,

    pub op_code: OpCode,
    pub op_rand: [u8; 2],
    pub cycle: u8,
    pub addr_mode: Addressing,

    pub rst: bool,
    pub nmi: bool,
    pub irq: bool,

    pub cpu_run: bool,
    pub nes_mem: NESMemory,
}


impl RP2A03{
    pub fn new() -> Self {
        RP2A03 {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_p: R_FLG, // ビット5: Reaerved.予約済 (常に1固定)
            reg_sp: 0xFF,
            reg_pc: 0x0600,
            // reg_pc: ADDR_PRG_ROM,
            // reg_pc: ADDR_VEC_TBL_RST,

            op_code: OpCode::NOP,
            op_rand: [0; 2],
            cycle: 0,
            addr_mode: Addressing::IMPL,

            cpu_run: false,
            rst: false,
            nmi: false,
            irq: false,
            nes_mem: NESMemory::new(),
        }
    }

    fn cls_status_flg(&mut self, flg: u8) {
        self.reg_p &= !flg;
    }

    fn set_status_flg(&mut self, flg: u8) {
        self.reg_p |= flg;
    }

    fn get_status_flg(&self, flg: u8) -> bool {
        (self.reg_p & flg) != 0
    }

    fn get_status_flg_all(&self) -> u8 {
        self.reg_p
    }

    fn set_status_flg_all(&mut self, val: u8) {
        self.reg_p = val;
    }

    fn cls_status_flg_all(&mut self) {
        self.reg_p = R_FLG;
    }

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
        let ret: u8 = val_a.wrapping_add(val_b);
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

        ret = ret << 1;
        if ret >  0x00FF {
            ret = ret & 0x00FF;
        }
        if (ret & (BIN_BIT_7 as u16)) != 0 {
            self.set_status_flg(CARRY_FLG);
        }else {
            self.cls_status_flg(CARRY_FLG);
        }
        ret as u8
    }

    fn c_flg_update_r_shit(&mut self, val: u8) -> u8{
        let mut ret: u16 = val as u16;
        ret = ret >> 1;
        if ret <= 0x00 {
            ret = 0;
        }
        if (ret & (BIN_BIT_0 as u16)) != 0 {
            self.set_status_flg(CARRY_FLG);
        }else {
            self.cls_status_flg(CARRY_FLG);
        }
        ret as u8
    }

    fn reset(&mut self){
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.cls_status_flg_all();
        self.reg_sp = 0xFF;
        self.set_status_flg(INTERRUPT_DISABLE_FLG);
        self.cpu_run = true;

        // (DEBUG) リセットベクタに飛ばず、PRG-ROMに
        // self.reg_pc = ADDR_PRG_ROM;
        self.interrupt_proc(InterruptType::RST);

        // // (DEBUG) ダーミープログラム用に
        // self.set_status_flg(OVERFLOW_FLG);

        // // (DEBUG) スネークゲーム用に
        self.reg_pc = 0x600;
    }

    fn interrupt_proc(&mut self, int_type :InterruptType)
    {
        match int_type {
            InterruptType::RST => {
                self.reg_pc = ADDR_VEC_TBL_RST;
            },
            InterruptType::NMI => {
                // TODO: NMI
                self.reg_pc = ADDR_VEC_TBL_NMI;
            },
            InterruptType::IRQ => {
                // TODO: NMI
                self.reg_pc = ADDR_VEC_TBL_IRQ;
            },
        }

        let addr_l: u16 = self.read(self.reg_pc) as u16;
        self.reg_pc += 1;
        let addr_u: u16 = self.read(self.reg_pc) as u16;
        self.reg_pc = (addr_u << 8) | addr_l;
    }

    fn read(&mut self, address: u16) -> u8
    {
        self.nes_mem.mem_read(address)
    }

    fn write(&mut self, address: u16, data: u8)
    {
        self.nes_mem.mem_write(address, data);
    }

    fn fetch_instruction(&mut self) -> u8 {
        let op_code = self.read(self.reg_pc);
        op_code
    }

    fn decode_instruction(&mut self, op_code: u8) {
        match op_code.into() {
            0x00 => { self.op_code = OpCode::BRK; self.addr_mode = Addressing::IMPL },
            0x01 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::IndX },
            0x05 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::ZPG },
            0x06 => { self.op_code = OpCode::ASL; self.addr_mode = Addressing::ZPG },
            0x08 => { self.op_code = OpCode::PHP; self.addr_mode = Addressing::IMPL },
            0x09 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::IMM },
            0x0A => { self.op_code = OpCode::ASL; self.addr_mode = Addressing::ACC },
            0x0D => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::ABS },
            0x0E => { self.op_code = OpCode::ASL; self.addr_mode = Addressing::ABS },
            0x10 => { self.op_code = OpCode::BPL; self.addr_mode = Addressing::REL },
            0x11 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::IndY },
            0x15 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::ZpgX },
            0x16 => { self.op_code = OpCode::ASL; self.addr_mode = Addressing::ZpgX },
            0x18 => { self.op_code = OpCode::CLC; self.addr_mode = Addressing::IMPL },
            0x19 => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::AbsY },
            0x1D => { self.op_code = OpCode::ORA; self.addr_mode = Addressing::AbsX },
            0x1E => { self.op_code = OpCode::ASL; self.addr_mode = Addressing::AbsX },
            0x20 => { self.op_code = OpCode::JSR; self.addr_mode = Addressing::ABS },
            0x21 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::IndX },
            0x24 => { self.op_code = OpCode::BIT; self.addr_mode = Addressing::ZPG },
            0x25 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::ZPG },
            0x26 => { self.op_code = OpCode::ROL; self.addr_mode = Addressing::ZPG },
            0x28 => { self.op_code = OpCode::PLP; self.addr_mode = Addressing::IMPL },
            0x29 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::IMM },
            0x2A => { self.op_code = OpCode::ROL; self.addr_mode = Addressing::ACC },
            0x2C => { self.op_code = OpCode::BIT; self.addr_mode = Addressing::ABS },
            0x2D => { self.op_code = OpCode::AND; self.addr_mode = Addressing::ABS },
            0x2E => { self.op_code = OpCode::ROL; self.addr_mode = Addressing::ABS },
            0x30 => { self.op_code = OpCode::BMI; self.addr_mode = Addressing::REL },
            0x31 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::IndY },
            0x35 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::ZpgX },
            0x36 => { self.op_code = OpCode::ROL; self.addr_mode = Addressing::ZpgX },
            0x38 => { self.op_code = OpCode::SEC; self.addr_mode = Addressing::IMPL },
            0x39 => { self.op_code = OpCode::AND; self.addr_mode = Addressing::AbsY },
            0x3D => { self.op_code = OpCode::AND; self.addr_mode = Addressing::AbsX },
            0x3E => { self.op_code = OpCode::ROL; self.addr_mode = Addressing::AbsX },
            0x40 => { self.op_code = OpCode::RTI; self.addr_mode = Addressing::IMPL },
            0x41 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::IndX },
            0x45 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::ZPG },
            0x46 => { self.op_code = OpCode::LSR; self.addr_mode = Addressing::ZPG },
            0x48 => { self.op_code = OpCode::PHA; self.addr_mode = Addressing::IMPL },
            0x49 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::IMM },
            0x4A => { self.op_code = OpCode::LSR; self.addr_mode = Addressing::ACC },
            0x4C => { self.op_code = OpCode::JMP; self.addr_mode = Addressing::ABS },
            0x4D => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::ABS },
            0x4E => { self.op_code = OpCode::LSR; self.addr_mode = Addressing::ABS },
            0x50 => { self.op_code = OpCode::BVC; self.addr_mode = Addressing::REL },
            0x51 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::IndY },
            0x55 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::ZpgX },
            0x56 => { self.op_code = OpCode::LSR; self.addr_mode = Addressing::ZpgX },
            0x58 => { self.op_code = OpCode::CLI; self.addr_mode = Addressing::IMPL },
            0x59 => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::AbsY },
            0x5D => { self.op_code = OpCode::EOR; self.addr_mode = Addressing::AbsX },
            0x5E => { self.op_code = OpCode::LSR; self.addr_mode = Addressing::AbsX },
            0x60 => { self.op_code = OpCode::RTS; self.addr_mode = Addressing::IMPL },
            0x61 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::IndX },
            0x65 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::ZPG },
            0x66 => { self.op_code = OpCode::ROR; self.addr_mode = Addressing::ZPG },
            0x68 => { self.op_code = OpCode::PLA; self.addr_mode = Addressing::IMPL },
            0x69 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::IMM },
            0x6A => { self.op_code = OpCode::ROR; self.addr_mode = Addressing::ACC },
            0x6C => { self.op_code = OpCode::JMP; self.addr_mode = Addressing::IND },
            0x6D => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::ABS },
            0x6E => { self.op_code = OpCode::ROR; self.addr_mode = Addressing::ABS },
            0x70 => { self.op_code = OpCode::BVS; self.addr_mode = Addressing::REL },
            0x71 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::IndY },
            0x75 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::ZpgX },
            0x76 => { self.op_code = OpCode::ROR; self.addr_mode = Addressing::ZpgX },
            0x78 => { self.op_code = OpCode::SEI; self.addr_mode = Addressing::IMPL },
            0x79 => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::AbsY },
            0x7D => { self.op_code = OpCode::ADC; self.addr_mode = Addressing::AbsX },
            0x7E => { self.op_code = OpCode::ROR; self.addr_mode = Addressing::AbsX },
            0x81 => { self.op_code = OpCode::STA; self.addr_mode = Addressing::IndX },
            0x84 => { self.op_code = OpCode::STY; self.addr_mode = Addressing::ZPG },
            0x85 => { self.op_code = OpCode::STA; self.addr_mode = Addressing::ZPG },
            0x86 => { self.op_code = OpCode::STX; self.addr_mode = Addressing::ZPG },
            0x88 => { self.op_code = OpCode::DEY; self.addr_mode = Addressing::IMPL },
            0x8A => { self.op_code = OpCode::TXA; self.addr_mode = Addressing::IMPL },
            0x8C => { self.op_code = OpCode::STY; self.addr_mode = Addressing::ABS },
            0x8D => { self.op_code = OpCode::STA; self.addr_mode = Addressing::ABS },
            0x8E => { self.op_code = OpCode::STX; self.addr_mode = Addressing::ABS },
            0x90 => { self.op_code = OpCode::BCC; self.addr_mode = Addressing::REL },
            0x91 => { self.op_code = OpCode::STA; self.addr_mode = Addressing::IndY },
            0x94 => { self.op_code = OpCode::STY; self.addr_mode = Addressing::ZpgX },
            0x95 => { self.op_code = OpCode::STA; self.addr_mode = Addressing::ZpgX },
            0x96 => { self.op_code = OpCode::STX; self.addr_mode = Addressing::ZpgY },
            0x98 => { self.op_code = OpCode::TYA; self.addr_mode = Addressing::IMPL },
            0x99 => { self.op_code = OpCode::STA; self.addr_mode = Addressing::AbsY },
            0x9A => { self.op_code = OpCode::TXS; self.addr_mode = Addressing::IMPL },
            0x9D => { self.op_code = OpCode::STA; self.addr_mode = Addressing::AbsX },
            0xA0 => { self.op_code = OpCode::LDY; self.addr_mode = Addressing::IMM },
            0xA1 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::IndX },
            0xA2 => { self.op_code = OpCode::LDX; self.addr_mode = Addressing::IMM },
            0xA4 => { self.op_code = OpCode::LDY; self.addr_mode = Addressing::ZPG },
            0xA5 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::ZPG },
            0xA6 => { self.op_code = OpCode::LDX; self.addr_mode = Addressing::ZPG },
            0xA8 => { self.op_code = OpCode::TAY; self.addr_mode = Addressing::IMPL },
            0xA9 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::IMM },
            0xAA => { self.op_code = OpCode::TAX; self.addr_mode = Addressing::IMPL },
            0xAC => { self.op_code = OpCode::LDY; self.addr_mode = Addressing::ABS },
            0xAD => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::ABS },
            0xAE => { self.op_code = OpCode::LDX; self.addr_mode = Addressing::ABS },
            0xB0 => { self.op_code = OpCode::BCS; self.addr_mode = Addressing::REL },
            0xB1 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::IndY },
            0xB4 => { self.op_code = OpCode::LDY; self.addr_mode = Addressing::ZpgX },
            0xB5 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::ZpgX },
            0xB6 => { self.op_code = OpCode::LDX; self.addr_mode = Addressing::ZpgY },
            0xB8 => { self.op_code = OpCode::CLV; self.addr_mode = Addressing::IMPL },
            0xB9 => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::AbsY },
            0xBA => { self.op_code = OpCode::TSX; self.addr_mode = Addressing::IMPL },
            0xBC => { self.op_code = OpCode::LDY; self.addr_mode = Addressing::AbsX },
            0xBD => { self.op_code = OpCode::LDA; self.addr_mode = Addressing::AbsX },
            0xBE => { self.op_code = OpCode::LDX; self.addr_mode = Addressing::AbsY },
            0xC0 => { self.op_code = OpCode::CPY; self.addr_mode = Addressing::IMM },
            0xC1 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::IndX },
            0xC4 => { self.op_code = OpCode::CPY; self.addr_mode = Addressing::ZPG },
            0xC5 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::ZPG },
            0xC6 => { self.op_code = OpCode::DEC; self.addr_mode = Addressing::ZPG },
            0xC8 => { self.op_code = OpCode::INY; self.addr_mode = Addressing::IMPL },
            0xC9 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::IMM },
            0xCA => { self.op_code = OpCode::DEX; self.addr_mode = Addressing::IMPL },
            0xCC => { self.op_code = OpCode::CPY; self.addr_mode = Addressing::ABS },
            0xCD => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::ABS },
            0xCE => { self.op_code = OpCode::DEC; self.addr_mode = Addressing::ABS },
            0xD0 => { self.op_code = OpCode::BNE; self.addr_mode = Addressing::REL },
            0xD1 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::IndY },
            0xD5 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::ZpgX },
            0xD6 => { self.op_code = OpCode::DEC; self.addr_mode = Addressing::ZpgX },
            0xD8 => { self.op_code = OpCode::CLD; self.addr_mode = Addressing::IMPL },
            0xD9 => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::AbsY },
            0xDD => { self.op_code = OpCode::CMP; self.addr_mode = Addressing::AbsX },
            0xDE => { self.op_code = OpCode::DEC; self.addr_mode = Addressing::AbsX },
            0xE0 => { self.op_code = OpCode::CPX; self.addr_mode = Addressing::IMM },
            0xE1 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::IndX },
            0xE4 => { self.op_code = OpCode::CPX; self.addr_mode = Addressing::ZPG },
            0xE5 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::ZPG },
            0xE6 => { self.op_code = OpCode::INC; self.addr_mode = Addressing::ZPG },
            0xE8 => { self.op_code = OpCode::INX; self.addr_mode = Addressing::IMPL },
            0xE9 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::IMM },
            0xEC => { self.op_code = OpCode::CPX; self.addr_mode = Addressing::ABS },
            0xED => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::ABS },
            0xEE => { self.op_code = OpCode::INC; self.addr_mode = Addressing::ABS },
            0xF0 => { self.op_code = OpCode::BEQ; self.addr_mode = Addressing::REL },
            0xF1 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::IndY },
            0xF5 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::ZpgX },
            0xF6 => { self.op_code = OpCode::INC; self.addr_mode = Addressing::ZpgX },
            0xF8 => { self.op_code = OpCode::SED; self.addr_mode = Addressing::IMPL },
            0xF9 => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::AbsY },
            0xFD => { self.op_code = OpCode::SBC; self.addr_mode = Addressing::AbsX },
            0xFE => { self.op_code = OpCode::INC; self.addr_mode = Addressing::AbsX },

            // NOP
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                self.op_code = OpCode::NOP; self.addr_mode = Addressing::IMPL },
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                self.op_code = OpCode::NOP; self.addr_mode = Addressing::IMM },
            0x04 | 0x44 | 0x64 => {
                self.op_code = OpCode::NOP; self.addr_mode = Addressing::ZPG },
            0x14 | 0x34 | 0x54 | 0x74| 0xD4| 0xF4 => {
                self.op_code = OpCode::NOP; self.addr_mode = Addressing::ZpgX },
            0x0C => { self.op_code = OpCode::NOP; self.addr_mode = Addressing::ABS },
            0x1C | 0x3C | 0x5C | 0x7C| 0xDC| 0xFC => {
                self.op_code = OpCode::NOP; self.addr_mode = Addressing::AbsX },

            // STP
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2  => {
                self.op_code = OpCode::STP; self.addr_mode = Addressing::IMPL },

            _ => { self.op_code = OpCode::UNK; self.addr_mode = Addressing::IMPL }
        };
    }

    fn execute_instruction(&mut self) {
        let (operand,operand_second,dbg_str) = self.read_operand();
        let mut jmp_flg: bool = false;

        match self.op_code {
            OpCode::NOP => {
                // No operation, do nothing
                println!("{}",format!("[DEBUG]: NOP ${}",dbg_str));
            }

            // // Logical Operations / 論理演算命令
            OpCode::AND => {
                println!("{}", format!("[DEBUG]: AND ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a & val;
                self.reg_a = result as u8;
                self.nzv_flg_update(result);
            }
            OpCode::ORA => {
                println!("{}", format!("[DEBUG]: ORA ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a | val;
                self.reg_a = result as u8;
                self.nzv_flg_update(result);
            }
            OpCode::EOR => {
                println!("{}", format!("[DEBUG]: EOR ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a ^ val;
                self.reg_a = result as u8;
                self.nzv_flg_update(result);
            }

            // Arithmetic Operations / 算術倫理演算
            OpCode::ADC => {
                println!("{}",format!("[DEBUG]: ADC ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u8 = value;
                    let mut carry: u8 = 0x00;
                    if self.get_status_flg(CARRY_FLG) {
                        carry = 0x01;
                    }
                    let mut ret: u8 = self.c_flg_update_add(self.reg_a, carry as u8 + val as u8);
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = self.c_flg_update_add(self.reg_a, carry as u8 + (((val2 as u16) << 8) | val as u16) as u8);
                    }
                    self.reg_a = ret;
                    self.nzv_flg_update(ret);
                }
            }
            OpCode::SBC => {
                println!("{}",format!("[DEBUG]: SBC ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u8 = value;
                    let mut carry: u8 = 0x01;
                    if self.get_status_flg(CARRY_FLG) {
                        carry = 0x00;
                    }
                    let mut ret: u8 = self.reg_a.wrapping_sub(val).wrapping_sub(carry) as u8;
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (self.reg_a.wrapping_sub((((val2 as u16) << 8) | val as u16) as u8).wrapping_sub(carry)) as u8;
                    }
                    self.reg_a = ret;
                    self.nzv_flg_update(ret);
                }
            }
            OpCode::CMP => {
                println!("{}",format!("[DEBUG]: CMP ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u8 = value;
                    let mut ret: u8 = val;
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }

                    if self.reg_a > ret {
                        self.set_status_flg(CARRY_FLG);
                    }
                    if self.reg_a == ret {
                        self.set_status_flg(CARRY_FLG);
                        self.set_status_flg(ZERO_FLG);
                    }
                    if self.reg_a < ret {
                    }
                    if (ret & BIN_BIT_7) != 0 {
                        self.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpCode::CPX => {
                println!("{}",format!("[DEBUG]: CPX ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u8 = value;
                    let mut ret: u8 = val;
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }

                    if self.reg_x > ret {
                        self.set_status_flg(CARRY_FLG);
                    }
                    if self.reg_x == ret {
                        self.set_status_flg(CARRY_FLG);
                        self.set_status_flg(ZERO_FLG);
                    }
                    if self.reg_x < ret {
                    }
                    if (ret & BIN_BIT_7) != 0 {
                        self.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpCode::CPY => {
                println!("{}",format!("[DEBUG]: CPY ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u8 = value;
                    let mut ret: u8 = val;
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }

                    if self.reg_y > ret {
                        self.set_status_flg(CARRY_FLG);
                    }
                    if self.reg_y == ret {
                        self.set_status_flg(CARRY_FLG);
                        self.set_status_flg(ZERO_FLG);
                    }
                    if self.reg_y < ret {
                    }
                    if (ret & BIN_BIT_7) != 0 {
                        self.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpCode::INC => {
                println!("{}",format!("[DEBUG]: INC ${}",dbg_str));
                let mut _addr: u16 = 0;
                let mut _ret: u8 = 0;
                if let Some(val) = operand {
                    _addr = val as u16;
                    if let Some(val2) = operand_second {
                        _addr = ((val2 as u16) << 8) | val as u16;
                    }
                    _ret = self.read(_addr);
                    _ret = self.c_flg_update_add(_ret, 0x01);
                    self.write(self.reg_pc, _ret);
                    self.nzv_flg_update(_ret as u8);
                }
            }
            OpCode::INX => {
                println!("{}",format!("[DEBUG]: INX ${}",dbg_str));
                let ret: u8 = self.c_flg_update_add(self.reg_x, 1);
                self.reg_x = ret;
                self.nzv_flg_update(ret);
            }
            OpCode::INY => {
                println!("{}",format!("[DEBUG]: INY ${}",dbg_str));
                let ret: u8 = self.c_flg_update_add(self.reg_y, 1);
                self.reg_y = ret;
                self.nzv_flg_update(ret);
            }
            OpCode::DEC => {
                println!("{}",format!("[DEBUG]: DEC ${}",dbg_str));
                let mut _addr: u16 = 0;
                let mut _ret: u8 = 0;
                if let Some(val) = operand {
                    _addr = val as u16;
                    if let Some(val2) = operand_second {
                        _addr = ((val2 as u16) << 8) | val as u16;
                    }
                    _ret = self.read(_addr).wrapping_sub(0x01);
                    self.write(self.reg_pc, _ret);
                    self.nzv_flg_update(_ret as u8);
                }
            }
            OpCode::DEX => {
                println!("{}",format!("[DEBUG]: DEX ${}",dbg_str));
                self.reg_x = self.reg_x.wrapping_sub(0x01);
                self.nzv_flg_update(self.reg_x);
            }
            OpCode::DEY => {
                println!("{}",format!("[DEBUG]: DEY ${}",dbg_str));
                self.reg_y = self.reg_y.wrapping_sub(0x01);
                self.nzv_flg_update(self.reg_y);
            }

            // Shift and Rotate Operations
            OpCode::ASL => {
                println!("{}",format!("[DEBUG]: ASL ${}",dbg_str));
                match self.addr_mode {
                    Addressing::ACC => {
                        let mut ret: u8 = self.c_flg_update_l_shit(self.reg_a);
                        ret = ret & 0xFE; // bit0, clear
                        self.nzv_flg_update(ret);
                        self.reg_a = ret;
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value;
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16);
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value;
                                let addr_u: u8 = value2;
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16);
                            }
                            let mut ret: u8 = self.c_flg_update_l_shit(val as u8);
                            ret = ret & 0xFE; // bit0, clear
                            self.nzv_flg_update(ret);
                            self.write(self.reg_pc, ret);
                        }
                    }
                }
            }
            OpCode::LSR => {
                println!("{}",format!("[DEBUG]: LSR ${}",dbg_str));
                match self.addr_mode {
                    Addressing::ACC => {
                        let mut ret: u8 = self.c_flg_update_r_shit(self.reg_a);
                        ret = ret & 0x7F; // bit7, clear
                        self.nzv_flg_update(ret);
                        self.reg_a = ret;
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value;
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16);
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value;
                                let addr_u: u8 = value2;
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16);
                            }
                            let mut ret: u8 = self.c_flg_update_r_shit(val as u8);
                            ret = ret & 0x7F; // bit7, clear
                            self.nzv_flg_update(ret);
                            self.write(self.reg_pc, ret);
                        }
                    }
                }
            }
            OpCode::ROL => {
                match self.addr_mode {
                    Addressing::ACC => {
                        println!("{}",format!("[DEBUG]: ROL ${}",dbg_str));
                        let mut ret: u8 = self.c_flg_update_l_shit(self.reg_a);
                        if self.get_status_flg(CARRY_FLG) {
                            ret = ret | BIN_BIT_0; // bit0 = C Flag Set
                        }else{
                            ret = ret & 0xFE; // bit0 = C Flag Clear
                        }
                        self.nzv_flg_update(ret);
                        self.reg_a = ret;
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value;
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16);
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value;
                                let addr_u: u8 = value2;
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16);
                            }

                            let mut ret: u8 = self.c_flg_update_l_shit(val as u8);
                            if self.get_status_flg(CARRY_FLG) {
                                ret = ret | BIN_BIT_0; // bit0 = C Flag Set
                            }else{
                                ret = ret & 0xFE; // bit0 = C Flag Clear
                            }
                            self.nzv_flg_update(ret);
                            self.write(self.reg_pc, ret);
                        }
                    }
                }
            }
            OpCode::ROR => {
                println!("{}",format!("[DEBUG]: ROR ${}",dbg_str));
                match self.addr_mode {
                    Addressing::ACC => {
                        let mut ret: u8 = self.c_flg_update_r_shit(self.reg_a);
                        if self.get_status_flg(CARRY_FLG) {
                            ret = ret | BIN_BIT_7; // bit7 = C Flag Set
                        }else{
                            ret = ret & 0x7F;      // bit7 = C Flag Clear
                        }
                        self.nzv_flg_update(ret);
                        self.reg_a = ret;
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value;
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16);
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value;
                                let addr_u: u8 = value2;
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16);
                            }

                            let mut ret: u8 = self.c_flg_update_r_shit(val as u8);
                            if self.get_status_flg(CARRY_FLG) {
                                ret = ret | BIN_BIT_7; // bit7 = C Flag Set
                            }else{
                                ret = ret & 0x7F;      // bit7 = C Flag Clear
                            }
                            self.nzv_flg_update(ret);
                            self.write(self.reg_pc, ret);
                        }
                    }
                }
            }

            // Load/Store Operations
            OpCode::LDA => {
                println!("{}",format!("[DEBUG]: LDA ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(val) = operand {
                    ret = self.read_operand_mem(val as u16);
                    if let Some(val2) = operand_second {
                        ret = self.read_operand_mem(((val2 as u16) << 8) | val as u16);
                    }
                }
                self.reg_a = ret;
                self.nzv_flg_update(ret);
            }
            OpCode::LDX => {
                println!("{}",format!("[DEBUG]: LDX ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(val) = operand {
                    ret = self.read_operand_mem(val as u16);
                    if let Some(val2) = operand_second {
                        ret = self.read_operand_mem(((val2 as u16) << 8) | val as u16);
                    }
                }
                self.reg_x = ret;
                self.nzv_flg_update(ret);
            }
            OpCode::LDY => {
                println!("{}",format!("[DEBUG]: LDY ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(val) = operand {
                    ret = self.read_operand_mem(val as u16);
                    if let Some(val2) = operand_second {
                        ret = self.read_operand_mem(((val2 as u16) << 8) | val as u16);
                    }
                }
                self.reg_y = ret;
                self.nzv_flg_update(ret);
            }
            OpCode::STA => {
                println!("{}",format!("[DEBUG]: STA ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(val) = operand {
                    addr = val as u16;
                    if let Some(val2) = operand_second {
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_a);
            }
            OpCode::STX => {
                println!("{}",format!("[DEBUG]: STX ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(val) = operand {
                    addr = val as u16;
                    if let Some(val2) = operand_second {
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_x);
            }
            OpCode::STY => {
                println!("{}",format!("[DEBUG]: STY ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(val) = operand {
                    addr = val as u16;
                    if let Some(val2) = operand_second {
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_y);
            }

            // Register Transfer Operations/レジスタ転送関連の命令
            OpCode::TAX => {
                println!("{}",format!("[DEBUG]: TAX ${}",dbg_str));
                self.reg_x = self.reg_a;
            }
            OpCode::TAY => {
                println!("{}",format!("[DEBUG]: TAY ${}",dbg_str));
                self.reg_y = self.reg_a;
            }
            OpCode::TXA => {
                println!("{}",format!("[DEBUG]: TXA ${}",dbg_str));
                self.reg_a = self.reg_x;
            }
            OpCode::TYA => {
                println!("{}",format!("[DEBUG]: TYA ${}",dbg_str));
                self.reg_a = self.reg_y;
            }

            // Stack Operations / スタック関連の命令
            OpCode::TSX => {
                println!("{}",format!("[DEBUG]: TSX ${}",dbg_str));
                self.reg_x = self.reg_sp;
            }
            OpCode::TXS => {
                println!("{}",format!("[DEBUG]: TXS ${}",dbg_str));
                self.reg_sp = self.reg_x;
            }
            OpCode::PHA => {
                println!("{}",format!("[DEBUG]: PHA ${}",dbg_str));
                self.push_stack(self.reg_a);
            }
            OpCode::PHP => {
                println!("{}",format!("[DEBUG]: PHP ${}",dbg_str));
                self.push_stack(self.reg_p);
            }
            OpCode::PLA => {
                println!("{}",format!("[DEBUG]: PLA ${}",dbg_str));
                let value = self.pop_stack();
                self.reg_a = value;
                self.nzv_flg_update(value);
            }
            OpCode::PLP => {
                println!("{}",format!("[DEBUG]: PLP ${}",dbg_str));
                let value = self.pop_stack();
                self.set_status_flg_all(value);
            }

            // Status Flag Operations / ステータスフラグ関連の命令
            OpCode::BIT => {
                println!("{}",format!("[DEBUG]: BIT ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(val1) = operand {
                    addr = val1 as u16;
                    if let Some(val2) = operand_second {
                        addr = (val2 as u16) << 8 | val1 as u16;
                    }
                    let ret: u8 = self.read_operand_mem(addr);
                    let result = self.reg_a & ret;
                    if result == 0 {
                        self.set_status_flg(ZERO_FLG);
                    }
                    if (ret & BIN_BIT_7) != 0 {
                        self.set_status_flg(NEGATIVE_FLG);
                    }
                    if (ret & BIN_BIT_6) != 0 {
                        self.set_status_flg(OVERFLOW_FLG);
                    }
                }
            }
            OpCode::CLC => {
                self.cls_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: CLC ${}",dbg_str));
            }
            OpCode::CLD => {
                self.cls_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: CLD ${}",dbg_str));
            }
            OpCode::CLI => {
                self.cls_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: CLI ${}",dbg_str));
            }
            OpCode::CLV => {
                self.cls_status_flg(OVERFLOW_FLG);
                println!("{}",format!("[DEBUG]: CLV ${}",dbg_str));
            }
            OpCode::SEC => {
                self.set_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: SEC ${}",dbg_str));
            }
            OpCode::SED => {
                self.set_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: SED ${}",dbg_str));
            }
            OpCode::SEI => {
                self.set_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: SEI ${}",dbg_str));
            }

            // Branch Operations / 分岐命令
            OpCode::BCC => {
                println!("{}",format!("[DEBUG]: BCC ${}",dbg_str));
                if self.get_status_flg(CARRY_FLG) != true {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BCS => {
                println!("{}",format!("[DEBUG]: BCS ${}",dbg_str));
                if self.get_status_flg(CARRY_FLG) != false {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BEQ => {
                println!("{}",format!("[DEBUG]: BEQ ${}",dbg_str));
                if self.get_status_flg(ZERO_FLG) != false {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BNE => {
                println!("{}",format!("[DEBUG]: BNE ${}",dbg_str));
                if self.get_status_flg(ZERO_FLG) != true {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BVC => {
                println!("{}",format!("[DEBUG]: BVC ${}",dbg_str));
                if self.get_status_flg(OVERFLOW_FLG) != true {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BVS => {
                println!("{}",format!("[DEBUG]: BVS ${}",dbg_str));
                if self.get_status_flg(OVERFLOW_FLG) != false {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BPL => {
                println!("{}",format!("[DEBUG]: BPL ${}",dbg_str));
                if self.get_status_flg(NEGATIVE_FLG) != true {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpCode::BMI => {
                println!("{}",format!("[DEBUG]: BMI ${}",dbg_str));
                if self.get_status_flg(NEGATIVE_FLG) != false {
                    if let Some(val1) = operand {
                        if let Some(val2) = operand_second {
                            let branch_addr: u16 =(val2 as u16) << 8 | val1 as u16;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }

            // Jump and Call Operations
            OpCode::JMP => {
                println!("{}",format!("[DEBUG]: JMP ${}",dbg_str));
                if let Some(val) = operand {
                    if let Some(val2) = operand_second {
                        let jump_addr: u16 = (val2 as u16) << 8 | val as u16;
                        self.reg_pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            OpCode::JSR => {
                println!("{}",format!("[DEBUG]: JSR {}",dbg_str));
                let mut jump_addr: u16 = 0x00;
                let return_addr: u16 = self.reg_pc;
                // let return_addr: u16 = self.reg_pc + 1;
                self.push_stack((return_addr & 0x00FF) as u8);
                self.push_stack(((return_addr & 0xFF00) >> 0x0008) as u8);

                if let Some(val) = operand {
                    if let Some(val2) = operand_second {
                        jump_addr = (val2 as u16) << 8 | val as u16;
                        self.reg_pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            OpCode::RTS => {
                println!("{}",format!("[DEBUG]: RTS ${}",dbg_str));
                let addr_u: u8 = self.pop_stack();
                let addr_l: u8 = self.pop_stack();
                let return_addr: u16 =(addr_u as u16) << 8 | addr_l as u16;
                self.reg_pc = return_addr + 1;
                jmp_flg = true;
            }

            // Intrrupt Operations / 割込み関連
            OpCode::RTI => {
                println!("{}",format!("[DEBUG]: RTI ${}",dbg_str));
                let status = self.pop_stack();
                self.set_status_flg_all(status.into());
                let addr_u: u8 = self.pop_stack();
                let addr_l: u8 = self.pop_stack();
                let return_addr: u16 =(addr_u as u16) << 8 | addr_l as u16;
                self.reg_pc = return_addr;
                jmp_flg = true;
            }
            OpCode::BRK => {
                println!("{}",format!("[DEBUG]: BRK ${}",dbg_str));
                if self.get_status_flg(BREAK_COMMAND_FLG) != true {
                    self.reg_pc += 1;
                    self.set_status_flg(BREAK_COMMAND_FLG);
                    self.push_stack((self.reg_pc & 0x00FF) as u8);
                    self.push_stack(((self.reg_pc & 0xFF00) >> 0x0008) as u8);
                    self.push_stack(self.get_status_flg_all());
                    self.set_status_flg(BREAK_COMMAND_FLG);
                    let mut _jmp_addr: u16 = self.read(ADDR_VEC_TBL_IRQ) as u16;
                    _jmp_addr |= (self.read(ADDR_VEC_TBL_IRQ + 1) as u16) << 0x0008;
                    self.reg_pc = _jmp_addr;
                    jmp_flg = true;
                    print!("BRK Jmp to: ${:04X}", self.reg_pc);
                }
            }

            // Other
            OpCode::STP | _ => {
                // TODO STPと未定義命令をどうするか
                println!("[DEBUG]: Undefined Instruction!");
            }
        }

        // pc ++
        if (operand != None) & (jmp_flg != true)
        {
            self.reg_pc += 1;
        }
    }

    fn push_stack(&mut self, data: u8) {
        let address: u16 = 0x0100u16.wrapping_add(self.reg_sp as u16);
        self.write(address, data);
        self.reg_sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.reg_sp += 1;
        let address: u16 = 0x0100u16.wrapping_add(self.reg_sp as u16);
        self.read(address)
    }

    fn read_operand(&mut self) -> (Option<u8>, Option<u8>, String)
    {
        self.reg_pc += 1;
        let oprand:u8 = self.read(self.reg_pc);

        match self.addr_mode {
            Addressing::ACC => {
                let acc:u8 = self.reg_a;
                (Some(self.reg_a),
                None,
                format!("{:#02X} (ACC)", acc))
            }
            Addressing::IMM => {
                (Some(self.read(self.reg_pc)),
                None,
                format!("{:#02X} (IMM)",oprand))
            }
            Addressing::ZPG => {
                (Some(self.read(self.reg_pc)),
                None,
                format!("{:#02X} (ZPG)",oprand))
            }
            Addressing::ZpgX => {
                let address: u16 = self.read(self.reg_pc.wrapping_add(self.reg_x as u16)) as u16;
                (Some(self.read(address)),
                None,
                format!("{:#02X} (ZpgX)",oprand))
            }
            Addressing::ZpgY => {
                let address = self.read(self.reg_pc.wrapping_add(self.reg_y as u16)) as u16;
                (Some(self.read(address)),
                None,
                format!("{:#02X} (ZpgY)",oprand))
            }
            Addressing::ABS => {
                let addr_l:u16 = self.read(self.reg_pc) as u16;
                self.reg_pc += 1;
                let addr_u:u16 = self.read(self.reg_pc) as u16;
                (Some(addr_l as u8),
                Some(addr_u as u8),
                format!("{:#02X} {:#02X} (ABS)",addr_l, addr_u))
            }
            Addressing::AbsX => {
                let mut addr_l: u16 = self.read(self.reg_pc) as u16;
                addr_l |= self.reg_x as u16;
                self.reg_pc += 1;
                let mut addr_u: u16 = self.read(self.reg_pc) as u16;
                addr_u |= addr_l;
                (Some(addr_l as u8),
                Some(addr_u as u8),
                format!("{:#02X} {:#02X} (AbsX)",addr_l, addr_u))
            }
            Addressing::AbsY => {
                let mut addr_l: u16 = self.read(self.reg_pc) as u16;
                addr_l |= self.reg_y as u16;
                self.reg_pc += 1;
                let mut addr_u: u16 = self.read(self.reg_pc) as u16;
                addr_u |= addr_l;
                (Some(addr_l as u8),
                Some(addr_u as u8),
                format!("{:#02X} {:#02X} (AbsY)",addr_l, addr_u))
            }
            Addressing::IND => {
                let addr_l: u16 = self.read(self.reg_pc) as u16;
                let addr_u: u16 = (addr_l & 0xff00) | (addr_l as u8).wrapping_add(1) as u16;
                (Some(addr_l as u8),
                Some(addr_u as u8),
                format!("{:#02X} (IND)",oprand))
            }
            Addressing::IndX => {
                let base_address: u16 = self.read(self.reg_pc.wrapping_add(self.reg_x as u16)) as u16;
                let address: u16 = self.read(base_address) as u16;
                (Some(self.read(address)),
                None,
                format!("{:#02X} (IndX)",oprand))
            }
            Addressing::IndY => {
                let base_address: u16 = self.read(self.reg_pc.wrapping_add(self.reg_y as u16)) as u16;
                let address: u8 = self.read(base_address);
                (Some(self.read(address as u16)),
                None,
                format!("{:#02X} (IndY)",oprand))
            }
            Addressing::REL => { // Relative Addressing(相対アドレッシング)
                let offset: u8 = self.read(self.reg_pc);
                let s_offset: i8 = offset as i8;
                let addr: u16 = (self.reg_pc as i16 + 1).wrapping_add(s_offset as i16) as u16;
                let addr_l: u8 = addr as u8;
                let addr_u: u8 = (addr >> 8) as u8;
                (Some(addr_l as u8),
                Some(addr_u as u8),
                format!("${:04X} (REL)(Offset: 0x{:02X}({}))", addr, s_offset, s_offset))
            }
            Addressing::IMPL => { // Implied Addressing
                // Not, Have Operand
                (None, None,format!("(IMPL)"))
            }
        }
    }

    fn read_operand_mem(&mut self, addr: u16) -> u8
    {
        match self.addr_mode {
            Addressing::ACC | Addressing::IMM => {
                addr as u8
            },
            Addressing::ZPG | Addressing::ZpgX | Addressing::ZpgY |
            Addressing::ABS | Addressing::AbsX | Addressing::AbsY |
            Addressing::IND | Addressing::IndX | Addressing::IndY  => {
                self.read(addr)
            },
            _ => {
                0
            }
        }
    }
}

fn cpu_reg_show()
{
    unsafe {
        let mut cpu = Pin::into_inner_unchecked(Pin::clone(&*S_CPU));
        println!("[DEBUG]: A:0x{:02X},X:0x{:02X},Y:0x{:02X},S:0x{:02X},P:{:08b},PC:0x{:04X}",
        cpu.reg_a,
        cpu.reg_x,
        cpu.reg_y,
        cpu.reg_sp,
        cpu.reg_p,
        cpu.reg_pc);
    }
}

static mut S_CPU: Lazy<Pin<Box<RP2A03>>> = Lazy::new(|| {
    let cpu = Box::pin(RP2A03::new());
    cpu
});

fn cpu_proc() {
    unsafe {
        let val = S_CPU.fetch_instruction();
        S_CPU.decode_instruction(val);
        S_CPU.execute_instruction();
    }
}

pub fn cpu_run(flg: bool) {
    unsafe {
        if flg {
            S_CPU.as_mut().cpu_run = true;
        } else {
            println!("[DEBUG]: CPU Stop");
            S_CPU.as_mut().cpu_run = false;
        }
    }
}

pub fn cpu_reset() -> Box<RP2A03> {
    unsafe {
        S_CPU.nes_mem.mem_reset();
        S_CPU.reset();
        cpu_run(true);
        let cpu_box: Box<RP2A03> = Box::from_raw(Pin::as_mut(&mut *S_CPU).get_mut());
        cpu_box
    }
}

pub fn cpu_main() {
    cpu_reg_show();
    cpu_proc();
}

// ====================================== TEST ======================================
#[cfg(test)]
mod cpu_test {

    #[test]
    fn cpu_test() {
        // TODO :CPU Test
    }
}
// ==================================================================================