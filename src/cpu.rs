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
pub enum Opcode {
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

    pub op_code: Opcode,
    pub op_rand: [u8; 2],
    pub cycle: u8,
    pub adressing: Addressing,

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

            op_code: Opcode::NOP,
            op_rand: [0; 2],
            cycle: 0,
            adressing: Addressing::IMPL,

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

    // fn cls_status_flg_all(&mut self) {
    //     self.reg_p = R_FLG;
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
        ret = ret.wrapping_add(val_b as u16);
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
        self.reg_p = 0;
        self.reg_sp = 0xFF;
        // self.reg_pc = ADDR_PRG_ROM;
        // self.reg_pc = ADDR_VEC_TBL_RST;
        self.set_status_flg(INTERRUPT_DISABLE_FLG);
        self.cpu_run = true;
        // self.interrupt_proc(InterruptType::RST);

        // (DEBUG) リセットベクタに飛ばず、PRG-ROMに
        // self.reg_pc = ADDR_PRG_ROM;

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

    fn decode_instruction(&mut self, op_code: u8) -> (Opcode, Addressing) {
        let mut opcode_type: Opcode = Opcode::NOP;
        let mut addr_mode: Addressing = Addressing::IMPL;

        match op_code.into() {
            0x00 => { opcode_type = Opcode::BRK; addr_mode = Addressing::IMPL },
            0x01 => { opcode_type = Opcode::ORA; addr_mode = Addressing::IndX },
            0x05 => { opcode_type = Opcode::ORA; addr_mode = Addressing::ZPG },
            0x06 => { opcode_type = Opcode::ASL; addr_mode = Addressing::ZPG },
            0x08 => { opcode_type = Opcode::PHP; addr_mode = Addressing::IMPL },
            0x09 => { opcode_type = Opcode::ORA; addr_mode = Addressing::IMM },
            0x0A => { opcode_type = Opcode::ASL; addr_mode = Addressing::ACC },
            0x0D => { opcode_type = Opcode::ORA; addr_mode = Addressing::ABS },
            0x0E => { opcode_type = Opcode::ASL; addr_mode = Addressing::ABS },
            0x10 => { opcode_type = Opcode::BPL; addr_mode = Addressing::REL },
            0x11 => { opcode_type = Opcode::ORA; addr_mode = Addressing::IndY },
            0x15 => { opcode_type = Opcode::ORA; addr_mode = Addressing::ZpgX },
            0x16 => { opcode_type = Opcode::ASL; addr_mode = Addressing::ZpgX },
            0x18 => { opcode_type = Opcode::CLC; addr_mode = Addressing::IMPL },
            0x19 => { opcode_type = Opcode::ORA; addr_mode = Addressing::AbsY },
            0x1D => { opcode_type = Opcode::ORA; addr_mode = Addressing::AbsX },
            0x1E => { opcode_type = Opcode::ASL; addr_mode = Addressing::AbsX },
            0x20 => { opcode_type = Opcode::JSR; addr_mode = Addressing::ABS },
            0x21 => { opcode_type = Opcode::AND; addr_mode = Addressing::IndX },
            0x24 => { opcode_type = Opcode::BIT; addr_mode = Addressing::ZPG },
            0x25 => { opcode_type = Opcode::AND; addr_mode = Addressing::ZPG },
            0x26 => { opcode_type = Opcode::ROL; addr_mode = Addressing::ZPG },
            0x28 => { opcode_type = Opcode::PLP; addr_mode = Addressing::IMPL },
            0x29 => { opcode_type = Opcode::AND; addr_mode = Addressing::IMM },
            0x2A => { opcode_type = Opcode::ROL; addr_mode = Addressing::ACC },
            0x2C => { opcode_type = Opcode::BIT; addr_mode = Addressing::ABS },
            0x2D => { opcode_type = Opcode::AND; addr_mode = Addressing::ABS },
            0x2E => { opcode_type = Opcode::ROL; addr_mode = Addressing::ABS },
            0x30 => { opcode_type = Opcode::BMI; addr_mode = Addressing::REL },
            0x31 => { opcode_type = Opcode::AND; addr_mode = Addressing::IndY },
            0x35 => { opcode_type = Opcode::AND; addr_mode = Addressing::ZpgX },
            0x36 => { opcode_type = Opcode::ROL; addr_mode = Addressing::ZpgX },
            0x38 => { opcode_type = Opcode::SEC; addr_mode = Addressing::IMPL },
            0x39 => { opcode_type = Opcode::AND; addr_mode = Addressing::AbsY },
            0x3D => { opcode_type = Opcode::AND; addr_mode = Addressing::AbsX },
            0x3E => { opcode_type = Opcode::ROL; addr_mode = Addressing::AbsX },
            0x40 => { opcode_type = Opcode::RTI; addr_mode = Addressing::IMPL },
            0x41 => { opcode_type = Opcode::EOR; addr_mode = Addressing::IndX },
            0x45 => { opcode_type = Opcode::EOR; addr_mode = Addressing::ZPG },
            0x46 => { opcode_type = Opcode::LSR; addr_mode = Addressing::ZPG },
            0x48 => { opcode_type = Opcode::PHA; addr_mode = Addressing::IMPL },
            0x49 => { opcode_type = Opcode::EOR; addr_mode = Addressing::IMM },
            0x4A => { opcode_type = Opcode::LSR; addr_mode = Addressing::ACC },
            0x4C => { opcode_type = Opcode::JMP; addr_mode = Addressing::ABS },
            0x4D => { opcode_type = Opcode::EOR; addr_mode = Addressing::ABS },
            0x4E => { opcode_type = Opcode::LSR; addr_mode = Addressing::ABS },
            0x50 => { opcode_type = Opcode::BVC; addr_mode = Addressing::REL },
            0x51 => { opcode_type = Opcode::EOR; addr_mode = Addressing::IndY },
            0x55 => { opcode_type = Opcode::EOR; addr_mode = Addressing::ZpgX },
            0x56 => { opcode_type = Opcode::LSR; addr_mode = Addressing::ZpgX },
            0x58 => { opcode_type = Opcode::CLI; addr_mode = Addressing::IMPL },
            0x59 => { opcode_type = Opcode::EOR; addr_mode = Addressing::AbsY },
            0x5D => { opcode_type = Opcode::EOR; addr_mode = Addressing::AbsX },
            0x5E => { opcode_type = Opcode::LSR; addr_mode = Addressing::AbsX },
            0x60 => { opcode_type = Opcode::RTS; addr_mode = Addressing::IMPL },
            0x61 => { opcode_type = Opcode::ADC; addr_mode = Addressing::IndX },
            0x65 => { opcode_type = Opcode::ADC; addr_mode = Addressing::ZPG },
            0x66 => { opcode_type = Opcode::ROR; addr_mode = Addressing::ZPG },
            0x68 => { opcode_type = Opcode::PLA; addr_mode = Addressing::IMPL },
            0x69 => { opcode_type = Opcode::ADC; addr_mode = Addressing::IMM },
            0x6A => { opcode_type = Opcode::ROR; addr_mode = Addressing::ACC },
            0x6C => { opcode_type = Opcode::JMP; addr_mode = Addressing::IND },
            0x6D => { opcode_type = Opcode::ADC; addr_mode = Addressing::ABS },
            0x6E => { opcode_type = Opcode::ROR; addr_mode = Addressing::ABS },
            0x70 => { opcode_type = Opcode::BVS; addr_mode = Addressing::REL },
            0x71 => { opcode_type = Opcode::ADC; addr_mode = Addressing::IndY },
            0x75 => { opcode_type = Opcode::ADC; addr_mode = Addressing::ZpgX },
            0x76 => { opcode_type = Opcode::ROR; addr_mode = Addressing::ZpgX },
            0x78 => { opcode_type = Opcode::SEI; addr_mode = Addressing::IMPL },
            0x79 => { opcode_type = Opcode::ADC; addr_mode = Addressing::AbsY },
            0x7D => { opcode_type = Opcode::ADC; addr_mode = Addressing::AbsX },
            0x7E => { opcode_type = Opcode::ROR; addr_mode = Addressing::AbsX },
            0x81 => { opcode_type = Opcode::STA; addr_mode = Addressing::IndX },
            0x84 => { opcode_type = Opcode::STY; addr_mode = Addressing::ZPG },
            0x85 => { opcode_type = Opcode::STA; addr_mode = Addressing::ZPG },
            0x86 => { opcode_type = Opcode::STX; addr_mode = Addressing::ZPG },
            0x88 => { opcode_type = Opcode::DEY; addr_mode = Addressing::IMPL },
            0x8A => { opcode_type = Opcode::TXA; addr_mode = Addressing::IMPL },
            0x8C => { opcode_type = Opcode::STY; addr_mode = Addressing::ABS },
            0x8D => { opcode_type = Opcode::STA; addr_mode = Addressing::ABS },
            0x8E => { opcode_type = Opcode::STX; addr_mode = Addressing::ABS },
            0x90 => { opcode_type = Opcode::BCC; addr_mode = Addressing::REL },
            0x91 => { opcode_type = Opcode::STA; addr_mode = Addressing::IndY },
            0x94 => { opcode_type = Opcode::STY; addr_mode = Addressing::ZpgX },
            0x95 => { opcode_type = Opcode::STA; addr_mode = Addressing::ZpgX },
            0x96 => { opcode_type = Opcode::STX; addr_mode = Addressing::ZpgY },
            0x98 => { opcode_type = Opcode::TYA; addr_mode = Addressing::IMPL },
            0x99 => { opcode_type = Opcode::STA; addr_mode = Addressing::AbsY },
            0x9A => { opcode_type = Opcode::TXS; addr_mode = Addressing::IMPL },
            0x9D => { opcode_type = Opcode::STA; addr_mode = Addressing::AbsX },
            0xA0 => { opcode_type = Opcode::LDY; addr_mode = Addressing::IMM },
            0xA1 => { opcode_type = Opcode::LDA; addr_mode = Addressing::IndX },
            0xA2 => { opcode_type = Opcode::LDX; addr_mode = Addressing::IMM },
            0xA4 => { opcode_type = Opcode::LDY; addr_mode = Addressing::ZPG },
            0xA5 => { opcode_type = Opcode::LDA; addr_mode = Addressing::ZPG },
            0xA6 => { opcode_type = Opcode::LDX; addr_mode = Addressing::ZPG },
            0xA8 => { opcode_type = Opcode::TAY; addr_mode = Addressing::IMPL },
            0xA9 => { opcode_type = Opcode::LDA; addr_mode = Addressing::IMM },
            0xAA => { opcode_type = Opcode::TAX; addr_mode = Addressing::IMPL },
            0xAC => { opcode_type = Opcode::LDY; addr_mode = Addressing::ABS },
            0xAD => { opcode_type = Opcode::LDA; addr_mode = Addressing::ABS },
            0xAE => { opcode_type = Opcode::LDX; addr_mode = Addressing::ABS },
            0xB0 => { opcode_type = Opcode::BCS; addr_mode = Addressing::REL },
            0xB1 => { opcode_type = Opcode::LDA; addr_mode = Addressing::IndY },
            0xB4 => { opcode_type = Opcode::LDY; addr_mode = Addressing::ZpgX },
            0xB5 => { opcode_type = Opcode::LDA; addr_mode = Addressing::ZpgX },
            0xB6 => { opcode_type = Opcode::LDX; addr_mode = Addressing::ZpgY },
            0xB8 => { opcode_type = Opcode::CLV; addr_mode = Addressing::IMPL },
            0xB9 => { opcode_type = Opcode::LDA; addr_mode = Addressing::AbsY },
            0xBA => { opcode_type = Opcode::TSX; addr_mode = Addressing::IMPL },
            0xBC => { opcode_type = Opcode::LDY; addr_mode = Addressing::AbsX },
            0xBD => { opcode_type = Opcode::LDA; addr_mode = Addressing::AbsX },
            0xBE => { opcode_type = Opcode::LDX; addr_mode = Addressing::AbsY },
            0xC0 => { opcode_type = Opcode::CPY; addr_mode = Addressing::IMM },
            0xC1 => { opcode_type = Opcode::CMP; addr_mode = Addressing::IndX },
            0xC4 => { opcode_type = Opcode::CPY; addr_mode = Addressing::ZPG },
            0xC5 => { opcode_type = Opcode::CMP; addr_mode = Addressing::ZPG },
            0xC6 => { opcode_type = Opcode::DEC; addr_mode = Addressing::ZPG },
            0xC8 => { opcode_type = Opcode::INY; addr_mode = Addressing::IMPL },
            0xC9 => { opcode_type = Opcode::CMP; addr_mode = Addressing::IMM },
            0xCA => { opcode_type = Opcode::DEX; addr_mode = Addressing::IMPL },
            0xCC => { opcode_type = Opcode::CPY; addr_mode = Addressing::ABS },
            0xCD => { opcode_type = Opcode::CMP; addr_mode = Addressing::ABS },
            0xCE => { opcode_type = Opcode::DEC; addr_mode = Addressing::ABS },
            0xD0 => { opcode_type = Opcode::BNE; addr_mode = Addressing::REL },
            0xD1 => { opcode_type = Opcode::CMP; addr_mode = Addressing::IndY },
            0xD5 => { opcode_type = Opcode::CMP; addr_mode = Addressing::ZpgX },
            0xD6 => { opcode_type = Opcode::DEC; addr_mode = Addressing::ZpgX },
            0xD8 => { opcode_type = Opcode::CLD; addr_mode = Addressing::IMPL },
            0xD9 => { opcode_type = Opcode::CMP; addr_mode = Addressing::AbsY },
            0xDD => { opcode_type = Opcode::CMP; addr_mode = Addressing::AbsX },
            0xDE => { opcode_type = Opcode::DEC; addr_mode = Addressing::AbsX },
            0xE0 => { opcode_type = Opcode::CPX; addr_mode = Addressing::IMM },
            0xE1 => { opcode_type = Opcode::SBC; addr_mode = Addressing::IndX },
            0xE4 => { opcode_type = Opcode::CPX; addr_mode = Addressing::ZPG },
            0xE5 => { opcode_type = Opcode::SBC; addr_mode = Addressing::ZPG },
            0xE6 => { opcode_type = Opcode::INC; addr_mode = Addressing::ZPG },
            0xE8 => { opcode_type = Opcode::INX; addr_mode = Addressing::IMPL },
            0xE9 => { opcode_type = Opcode::SBC; addr_mode = Addressing::IMM },
            0xEC => { opcode_type = Opcode::CPX; addr_mode = Addressing::ABS },
            0xED => { opcode_type = Opcode::SBC; addr_mode = Addressing::ABS },
            0xEE => { opcode_type = Opcode::INC; addr_mode = Addressing::ABS },
            0xF0 => { opcode_type = Opcode::BEQ; addr_mode = Addressing::REL },
            0xF1 => { opcode_type = Opcode::SBC; addr_mode = Addressing::IndY },
            0xF5 => { opcode_type = Opcode::SBC; addr_mode = Addressing::ZpgX },
            0xF6 => { opcode_type = Opcode::INC; addr_mode = Addressing::ZpgX },
            0xF8 => { opcode_type = Opcode::SED; addr_mode = Addressing::IMPL },
            0xF9 => { opcode_type = Opcode::SBC; addr_mode = Addressing::AbsY },
            0xFD => { opcode_type = Opcode::SBC; addr_mode = Addressing::AbsX },
            0xFE => { opcode_type = Opcode::INC; addr_mode = Addressing::AbsX },

            // NOP
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                opcode_type = Opcode::NOP; addr_mode = Addressing::IMPL },
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                opcode_type = Opcode::NOP; addr_mode = Addressing::IMM },
            0x04 | 0x44 | 0x64 => {
                opcode_type = Opcode::NOP; addr_mode = Addressing::ZPG },
            0x14 | 0x34 | 0x54 | 0x74| 0xD4| 0xF4 => {
                opcode_type = Opcode::NOP; addr_mode = Addressing::ZpgX },
            0x0C => { opcode_type = Opcode::NOP; addr_mode = Addressing::ABS },
            0x1C | 0x3C | 0x5C | 0x7C| 0xDC| 0xFC => {
                opcode_type = Opcode::NOP; addr_mode = Addressing::AbsX },

            // STP
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2  => {
                opcode_type = Opcode::STP; addr_mode = Addressing::IMPL },

            _ => { opcode_type = Opcode::UNK; addr_mode = Addressing::IMPL }
        };

        (opcode_type, addr_mode)
    }

    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing) {
        let _addressing: Addressing = addressing.clone();
        let (operand,operand_second,dbg_str) = self.read_operand(addressing);
        let mut jmp_flg: bool = false;

        match opcode {
            Opcode::NOP => {
                // No operation, do nothing
                println!("{}",format!("[DEBUG]: NOP ${}",dbg_str));
            }

            // // Logical Operations / 論理演算命令
            Opcode::AND => {
                println!("{}", format!("[DEBUG]: AND ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(&_addressing, value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(&_addressing, ((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a & val;
                self.reg_a = result as u8;
            }
            Opcode::ORA => {
                println!("{}", format!("[DEBUG]: ORA ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(&_addressing, value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(&_addressing, ((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a | val;
                self.reg_a = result as u8;
            }
            Opcode::EOR => {
                println!("{}", format!("[DEBUG]: EOR ${}", dbg_str));
                let mut result: u8 = 0;
                let mut val = 0;
                if let Some(value) = operand {
                    val = self.read_operand_mem(&_addressing, value as u16);
                    if let Some(value2) = operand_second {
                        val = self.read_operand_mem(&_addressing, ((value2 as u16) << 8) | value as u16);
                    }
                }
                result = self.reg_a ^ val;
                self.reg_a = result as u8;
            }

            // Arithmetic Operations / 算術倫理演算
            Opcode::ADC => {
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
            Opcode::SBC => {
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
            Opcode::CMP => {
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
            Opcode::CPX => {
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
            Opcode::CPY => {
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
            Opcode::INC => {
                println!("{}",format!("[DEBUG]: INC ${}",dbg_str));
                if let Some(value) = operand {
                    let val1: u8 = value;
                    let mut ret: u8 = self.c_flg_update_add(val1 as u8,1);
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = self.c_flg_update_add((((val2 as u16) << 8) | val1 as u16) as u8, 1);
                    }
                    self.write(self.reg_pc, ret);
                    self.nzv_flg_update(ret as u8);
                }
            }
            Opcode::INX => {
                println!("{}",format!("[DEBUG]: INX ${}",dbg_str));
                let ret: u8 = self.c_flg_update_add(self.reg_x, 1);
                self.reg_x = ret;
                self.nzv_flg_update(ret);
            }
            Opcode::INY => {
                println!("{}",format!("[DEBUG]: INY ${}",dbg_str));
                let ret: u8 = self.c_flg_update_add(self.reg_y, 1);
                self.reg_y = ret;
                self.nzv_flg_update(ret);
            }
            Opcode::DEC => {
                println!("{}",format!("[DEBUG]: DEC ${}",dbg_str));
                if let Some(value) = operand {
                    let val1: u8 = value;
                    let mut ret: u8 = val1.wrapping_sub(0x01);
                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val1 as u16).wrapping_sub(0x01) as u8;
                    }
                    self.write(self.reg_pc, ret);
                    self.nzv_flg_update(ret as u8);
                }
            }
            Opcode::DEX => {
                println!("{}",format!("[DEBUG]: DEX ${}",dbg_str));
                self.reg_x = self.reg_x.wrapping_sub(0x01);
                self.nzv_flg_update(self.reg_x);
            }
            Opcode::DEY => {
                println!("{}",format!("[DEBUG]: DEY ${}",dbg_str));
                self.reg_y = self.reg_y.wrapping_sub(0x01);
                self.nzv_flg_update(self.reg_y);
            }

            // Shift and Rotate Operations
            Opcode::ASL => {
                println!("{}",format!("[DEBUG]: ASL ${}",dbg_str));
                match _addressing {
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
            Opcode::LSR => {
                println!("{}",format!("[DEBUG]: LSR ${}",dbg_str));
                match _addressing {
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
            Opcode::ROL => {
                match _addressing {
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
            Opcode::ROR => {
                println!("{}",format!("[DEBUG]: ROR ${}",dbg_str));
                match _addressing {
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
            Opcode::LDA => {
                println!("{}",format!("[DEBUG]: LDA ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    ret = val;

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }
                }
                self.reg_a = ret;
            }
            Opcode::LDX => {
                println!("{}",format!("[DEBUG]: LDX ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    ret = val;

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }
                }
                self.reg_x = ret;
            }
            Opcode::LDY => {
                println!("{}",format!("[DEBUG]: LDY ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    ret = val;

                    if let Some(value2) = operand_second {
                        let val2: u8 = value2;
                        ret = (((val2 as u16) << 8) | val as u16) as u8;
                    }
                }
                self.reg_y = ret;
            }
            Opcode::STA => {
                println!("{}",format!("[DEBUG]: STA ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    addr = val as u16;
                    if let Some(value2) = operand_second {
                        let val: u8 = value;
                        let val2: u8 = value2;
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_a);
            }
            Opcode::STX => {
                println!("{}",format!("[DEBUG]: STX ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    addr = val as u16;
                    if let Some(value2) = operand_second {
                        let val: u8 = value;
                        let val2: u8 = value2;
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_x);
            }
            Opcode::STY => {
                println!("{}",format!("[DEBUG]: STY ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(value) = operand {
                    let val: u8 = value;
                    addr = val as u16;
                    if let Some(value2) = operand_second {
                        let val: u8 = value;
                        let val2: u8 = value2;
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, self.reg_y);
            }

            // Register Transfer Operations/レジスタ転送関連の命令
            Opcode::TAX => {
                println!("{}",format!("[DEBUG]: TAX ${}",dbg_str));
                self.reg_x = self.reg_a;
            }
            Opcode::TAY => {
                println!("{}",format!("[DEBUG]: TAY ${}",dbg_str));
                self.reg_y = self.reg_a;
            }
            Opcode::TXA => {
                println!("{}",format!("[DEBUG]: TXA ${}",dbg_str));
                self.reg_a = self.reg_x;
            }
            Opcode::TYA => {
                println!("{}",format!("[DEBUG]: TYA ${}",dbg_str));
                self.reg_a = self.reg_y;
            }

            // Stack Operations / スタック関連の命令
            Opcode::TSX => {
                println!("{}",format!("[DEBUG]: TSX ${}",dbg_str));
                self.reg_x = self.reg_sp;
            }
            Opcode::TXS => {
                println!("{}",format!("[DEBUG]: TXS ${}",dbg_str));
                self.reg_sp = self.reg_x;
            }
            Opcode::PHA => {
                println!("{}",format!("[DEBUG]: PHA ${}",dbg_str));
                self.push_stack(self.reg_a);
            }
            Opcode::PHP => {
                println!("{}",format!("[DEBUG]: PHP ${}",dbg_str));
                self.push_stack(self.reg_p);
            }
            Opcode::PLA => {
                println!("{}",format!("[DEBUG]: PLA ${}",dbg_str));
                let value = self.pop_stack();
                self.reg_a = value;
                self.nzv_flg_update(value);
            }
            Opcode::PLP => {
                println!("{}",format!("[DEBUG]: PLP ${}",dbg_str));
                let value = self.pop_stack();
                self.set_status_flg_all(value);
            }

            // Status Flag Operations / ステータスフラグ関連の命令
            Opcode::BIT => {
                println!("{}",format!("[DEBUG]: BIT ${}",dbg_str));
                let mut addr: u16 = 0;
                if let Some(value) = operand {
                    addr = value as u16;
                    if let Some(value2) = operand_second {
                        let addr_u: u16 = value2 as u16;
                        addr = addr_u << 8 | addr;
                    }
                    let ret: u8 = self.read_operand_mem(&_addressing ,addr);
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
            Opcode::CLC => {
                self.cls_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: CLC ${}",dbg_str));
            }
            Opcode::CLD => {
                self.cls_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: CLD ${}",dbg_str));
            }
            Opcode::CLI => {
                self.cls_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: CLI ${}",dbg_str));
            }
            Opcode::CLV => {
                self.cls_status_flg(OVERFLOW_FLG);
                println!("{}",format!("[DEBUG]: CLV ${}",dbg_str));
            }
            Opcode::SEC => {
                self.set_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: SEC ${}",dbg_str));
            }
            Opcode::SED => {
                self.set_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: SED ${}",dbg_str));
            }
            Opcode::SEI => {
                self.set_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: SEI ${}",dbg_str));
            }

            // Branch Operations / 分岐命令
            Opcode::BCC => {
                println!("{}",format!("[DEBUG]: BCC ${}",dbg_str));
                if self.get_status_flg(CARRY_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BCS => {
                println!("{}",format!("[DEBUG]: BCS ${}",dbg_str));
                if self.get_status_flg(CARRY_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BEQ => {
                println!("{}",format!("[DEBUG]: BEQ ${}",dbg_str));
                if self.get_status_flg(ZERO_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BNE => {
                println!("{}",format!("[DEBUG]: BNE ${}",dbg_str));
                if self.get_status_flg(ZERO_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BVC => {
                println!("{}",format!("[DEBUG]: BVC ${}",dbg_str));
                if self.get_status_flg(OVERFLOW_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BVS => {
                println!("{}",format!("[DEBUG]: BVS ${}",dbg_str));
                if self.get_status_flg(OVERFLOW_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BPL => {
                println!("{}",format!("[DEBUG]: BPL ${}",dbg_str));
                if self.get_status_flg(NEGATIVE_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            Opcode::BMI => {
                println!("{}",format!("[DEBUG]: BMI ${}",dbg_str));
                if self.get_status_flg(NEGATIVE_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value as u16;
                            let val2:u16 = value2 as u16;
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.reg_pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }

            // Jump and Call Operations
            Opcode::JMP => {
                println!("{}",format!("[DEBUG]: JMP ${}",dbg_str));
                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value;
                        let val2: u8 = value2;
                        let jump_addr: u16 = (val2 as u16) << 8 | val as u16;
                        self.reg_pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            Opcode::JSR => {
                println!("{}",format!("[DEBUG]: JSR {}",dbg_str));
                let mut jump_addr: u16 = 0x00;
                let return_addr: u16 = self.reg_pc;
                // let return_addr: u16 = self.reg_pc + 1;
                self.push_stack((return_addr & 0x00FF) as u8);
                self.push_stack(((return_addr & 0xFF00) >> 0x0008) as u8);

                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value;
                        let val2: u8 = value2;
                        jump_addr = (val2 as u16) << 8 | val as u16;
                        self.reg_pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            Opcode::RTS => {
                println!("{}",format!("[DEBUG]: RTS ${}",dbg_str));
                let return_addr_u: u16 = self.pop_stack() as u16;
                let return_addr_l: u16 = self.pop_stack() as u16;
                let return_addr: u16 = (return_addr_u << 8) | return_addr_l;
                self.reg_pc = return_addr + 1;
                jmp_flg = true;
            }

            // Intrrupt Operations / 割込み関連
            Opcode::RTI => {
                println!("{}",format!("[DEBUG]: RTI ${}",dbg_str));
                let status = self.pop_stack();
                self.set_status_flg_all(status.into());
                let return_addr_l: u16 = self.pop_stack() as u16;
                let return_addr_u: u16 = self.pop_stack() as u16;
                let return_addr: u16 = (return_addr_u << 8) | return_addr_l;
                self.reg_pc = return_addr;
                jmp_flg = true;
            }
            Opcode::BRK => {
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
            Opcode::STP | _ => {
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

    fn read_operand(&mut self, addressing: Addressing) -> (Option<u8>, Option<u8>, String)
    {
        self.reg_pc += 1;
        let oprand:u8 = self.read(self.reg_pc);

        match addressing {
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

    fn read_operand_mem(&mut self, addressing: &Addressing, addr: u16) -> u8
    {
        self.reg_pc += 1;
        let oprand:u8 = self.read(self.reg_pc);

        match addressing {
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
        let op_code = S_CPU.fetch_instruction();
        let (opcode, addressing) = S_CPU.decode_instruction(op_code);
        S_CPU.execute_instruction(opcode, addressing);
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