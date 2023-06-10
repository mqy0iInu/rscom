use crate::mem::*;
use std::convert::TryInto;
use std::convert::From;
use std::rc::Rc;
// use core::cell::OnceCell;
use std::cell::OnceCell;
use std::sync::{Once, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref S_CPU_DATA: Mutex<Option<RP2A03<u8>>> = Mutex::new(None);
    static ref S_CPU: Once = Once::new();
    static ref S_CPU_STOP: bool = false;
}

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

#[derive(Clone)]
pub struct ProgramCounter {
    pc: u16,
}

impl ProgramCounter {
    fn new() -> Self {
        ProgramCounter {
            pc : ADDR_VEC_TBL_RST, // リセットベクタ
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
    fn read_operand(&mut self, addressing: Addressing) -> (Option<T>, Option<T>, String);
    fn decode_instruction(&mut self, op_code: T) -> (Opcode, Addressing);
    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing);
    fn push_stack(&mut self, data: T);
    fn pop_stack(&mut self) -> T;
    fn interrupt_proc(&mut self, int_type :InterruptType);
}

pub const ADDR_VEC_TBL_RST: u16 = 0xFFFC;  // RESET Vector Table
pub const ADDR_VEC_TBL_NMI: u16 = 0xFFFA;  // NMI Vector Table
pub const ADDR_VEC_TBL_IRQ: u16 = 0xFFFE;  // IRQ Vector Table
enum InterruptType {
    RST,
    NMI,
    IRQ,
}

#[derive(Clone)]
pub struct Interrupt {
    rst: bool,
    nmi: bool,
    irq: bool,
}

impl Interrupt {
    fn new() -> Self {
        Interrupt {
            rst: true,
            nmi: false,
            irq: false
        }
    }
}

#[derive(Clone)]
/// RP2A03のステータスレジスタ
pub struct StatusRegister {
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
}

#[derive(Clone)]
pub struct RP2A03<T> {
    pub cpu_reg: [T; 4],
    pub cpu_p_reg: StatusRegister,
    pub cpu_pc: ProgramCounter,
    pub nes_mem: NESMemory,
    pub interrupt: Interrupt,
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

        // self.interrupt_proc(InterruptType::RST);

        // (DEBUG) リセットベクタに飛ばず、PRG-ROMに
        // self.cpu_pc.pc = ADDR_PRG_ROM;

        // // (DEBUG) ダーミープログラム用に
        // self.cpu_p_reg.set_status_flg(OVERFLOW_FLG);

        // // (DEBUG) スネークゲーム用に
        self.cpu_pc.pc = 0x600;
    }

    fn interrupt_proc(&mut self, int_type :InterruptType)
    {
        match int_type {
            InterruptType::RST => {
                self.cpu_pc.pc = ADDR_VEC_TBL_RST;
            },
            InterruptType::NMI => {
                // TODO: NMI
                self.cpu_pc.pc = ADDR_VEC_TBL_NMI;
            },
            InterruptType::IRQ => {
                // TODO: NMI
                self.cpu_pc.pc = ADDR_VEC_TBL_IRQ;
            },
        }

        let addr_l: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
        self.cpu_pc.pc += 1;
        let addr_u: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
        self.cpu_pc.pc = (addr_u << 8) | addr_l;
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
        let _addressing = addressing.clone();
        let (operand,operand_second,dbg_str) = self.read_operand(addressing);
        let mut jmp_flg: bool = false;

        match opcode.opcode_type {
            OpcodeType::NOP => {
                // No operation, do nothing
                println!("{}",format!("[DEBUG]: NOP ${}",dbg_str));
            }

            // // Logical Operations / 論理演算命令
            OpcodeType::AND => {
                println!("{}", format!("[DEBUG]: AND ${}", dbg_str));
                let a:u16 = self.get_register(CPUReg::A).try_into().unwrap();
                let mut result: u16 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    result = (a & val).try_into().unwrap();
                    if let Some(value2) = operand_second {
                        let val: u16 = value.try_into().unwrap();
                        let val2: u16 = value2.try_into().unwrap();
                        result = (a & ((val2 << 0x08) | val)).try_into().unwrap();
                    }
                }
                self.set_register(CPUReg::A, T::from(result as u8));
            }
            OpcodeType::ORA => {
                println!("{}", format!("[DEBUG]: ORA ${}", dbg_str));
                let a:u16 = self.get_register(CPUReg::A).try_into().unwrap();
                let mut result: u16 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    result = (a | val).try_into().unwrap();
                    if let Some(value2) = operand_second {
                        let val: u16 = value.try_into().unwrap();
                        let val2: u16 = value2.try_into().unwrap();
                        result = (a | ((val2 << 0x08) | val)).try_into().unwrap();
                    }
                }
                self.set_register(CPUReg::A, T::from(result as u8));
            }
            OpcodeType::EOR => {
                println!("{}", format!("[DEBUG]: EOR ${}", dbg_str));
                let a:u16 = self.get_register(CPUReg::A).try_into().unwrap();
                let mut result: u16 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    result = (a ^ val).try_into().unwrap();
                    if let Some(value2) = operand_second {
                        let val: u16 = value.try_into().unwrap();
                        let val2: u16 = value2.try_into().unwrap();
                        result = (a ^ ((val2 << 0x08) | val)).try_into().unwrap();
                    }
                }
                self.set_register(CPUReg::A, T::from(result as u8));
            }

            // Arithmetic Operations / 算術倫理演算
            OpcodeType::ADC => {
                println!("{}",format!("[DEBUG]: ADC ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    let a: u16 = self.get_register(CPUReg::A).try_into().unwrap();
                    let mut carry: u16 = 0x00;
                    if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                        carry = 0x01;
                    }
                    let mut ret: u8 = self.cpu_p_reg.c_flg_update_add(a as u8, carry as u8 + val as u8);
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = self.cpu_p_reg.c_flg_update_add(a as u8, carry as u8 + ((val2 << 8) | val) as u8);
                    }
                    self.set_register(CPUReg::A, T::from(ret as u8));
                    self.cpu_p_reg.nzv_flg_update(ret);
                }
            }
            OpcodeType::SBC => {
                println!("{}",format!("[DEBUG]: SBC ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    let a: u16 = self.get_register(CPUReg::A).try_into().unwrap();
                    let mut carry: u16 = 0x01;
                    if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                        carry = 0x00;
                    }
                    let mut ret: u8 = a.wrapping_sub(val).wrapping_sub(carry) as u8;
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = a.wrapping_sub((val2 << 8) | val).wrapping_sub(carry) as u8;
                    }
                    self.set_register(CPUReg::A, T::from(ret as u8));
                    self.cpu_p_reg.nzv_flg_update(ret);
                }
            }
            OpcodeType::CMP => {
                println!("{}",format!("[DEBUG]: CMP ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    let a: u16 = self.get_register(CPUReg::A).try_into().unwrap();
                    let mut ret: u16 = val;
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u16;
                    }

                    if a > ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                    }
                    if a == ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                        self.cpu_p_reg.set_status_flg(ZERO_FLG);
                    }
                    if a < ret {
                    }
                    if (ret & BIN_BIT_7 as u16) != 0 {
                        self.cpu_p_reg.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpcodeType::CPX => {
                println!("{}",format!("[DEBUG]: CPX ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    let x: u16 = self.get_register(CPUReg::X).try_into().unwrap();
                    let mut ret: u16 = val;
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u16;
                    }

                    if x > ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                    }
                    if x == ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                        self.cpu_p_reg.set_status_flg(ZERO_FLG);
                    }
                    if x < ret {
                    }
                    if (ret & BIN_BIT_7 as u16) != 0 {
                        self.cpu_p_reg.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpcodeType::CPY => {
                println!("{}",format!("[DEBUG]: CPY ${}",dbg_str));
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    let y: u16 = self.get_register(CPUReg::Y).try_into().unwrap();
                    let mut ret: u16 = val;
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u16;
                    }

                    if y > ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                    }
                    if y == ret {
                        self.cpu_p_reg.set_status_flg(CARRY_FLG);
                        self.cpu_p_reg.set_status_flg(ZERO_FLG);
                    }
                    if y < ret {
                    }
                    if (ret & BIN_BIT_7 as u16) != 0 {
                        self.cpu_p_reg.set_status_flg(NEGATIVE_FLG);
                    }
                }
            }
            OpcodeType::INC => {
                println!("{}",format!("[DEBUG]: INC ${}",dbg_str));
                if let Some(value) = operand {
                    let val1: u16 = value.try_into().unwrap();
                    let mut ret: u16 = self.cpu_p_reg.c_flg_update_add(val1 as u8,1) as u16;
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = self.cpu_p_reg.c_flg_update_add(((val2 << 8) | val1) as u8, 1) as u16;
                    }
                    self.write(self.cpu_pc.pc, T::from(ret as u8));
                    self.cpu_p_reg.nzv_flg_update(ret as u8);
                }
            }
            OpcodeType::INX => {
                println!("{}",format!("[DEBUG]: INX ${}",dbg_str));
                let x: u8 = self.get_register(CPUReg::X).try_into().unwrap();
                let ret: u8 = self.cpu_p_reg.c_flg_update_add(x, 1);
                self.set_register(CPUReg::X, T::from(ret as u8));
                self.cpu_p_reg.nzv_flg_update(ret);
            }
            OpcodeType::INY => {
                println!("{}",format!("[DEBUG]: INY ${}",dbg_str));
                let y: u8 = self.get_register(CPUReg::Y).try_into().unwrap();
                let ret: u8 = self.cpu_p_reg.c_flg_update_add(y, 1);
                self.set_register(CPUReg::Y, T::from(ret as u8));
                self.cpu_p_reg.nzv_flg_update(ret);
            }
            OpcodeType::DEC => {
                println!("{}",format!("[DEBUG]: DEC ${}",dbg_str));
                if let Some(value) = operand {
                    let val1: u16 = value.try_into().unwrap();
                    let mut ret: u16 = val1.wrapping_sub(0x01);
                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val1).wrapping_sub(0x01);
                    }
                    self.write(self.cpu_pc.pc, T::from(ret as u8));
                    self.cpu_p_reg.nzv_flg_update(ret as u8);
                }
            }
            OpcodeType::DEX => {
                println!("{}",format!("[DEBUG]: DEX ${}",dbg_str));
                let x: u8 = self.get_register(CPUReg::X).try_into().unwrap();
                let ret: u8 = x.wrapping_sub(0x01);
                self.set_register(CPUReg::X, ret.try_into().unwrap());
                self.cpu_p_reg.nzv_flg_update(ret);
            }
            OpcodeType::DEY => {
                println!("{}",format!("[DEBUG]: DEY ${}",dbg_str));
                let y: u8 = self.get_register(CPUReg::Y).try_into().unwrap();
                let ret: u8 = y.wrapping_sub(0x01);
                self.set_register(CPUReg::Y, ret.try_into().unwrap());
                self.cpu_p_reg.nzv_flg_update(ret);
            }

            // Shift and Rotate Operations
            OpcodeType::ASL => {
                println!("{}",format!("[DEBUG]: ASL ${}",dbg_str));
                match *_addressing.addr_mode {
                    AddrMode::ACC => {
                        let a: T = self.get_register(CPUReg::A);
                        let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(a.try_into().unwrap());
                        ret = ret & 0xFE; // bit0, clear
                        self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                        self.set_register(CPUReg::A, ret.try_into().unwrap());
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value.try_into().unwrap();
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16).try_into().unwrap();
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value.try_into().unwrap();
                                let addr_u: u8 = value2.try_into().unwrap();
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16).try_into().unwrap();
                            }
                            let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(val as u8);
                            ret = ret & 0xFE; // bit0, clear
                            self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                            self.write(self.cpu_pc.pc, T::from(ret as u8));
                        }
                    }
                }
            }
            OpcodeType::LSR => {
                println!("{}",format!("[DEBUG]: LSR ${}",dbg_str));
                match *_addressing.addr_mode {
                    AddrMode::ACC => {
                        let a: T = self.get_register(CPUReg::A);
                        let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(a.try_into().unwrap());
                        ret = ret & 0x7F; // bit7, clear
                        self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                        self.set_register(CPUReg::A, ret.try_into().unwrap());
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value.try_into().unwrap();
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16).try_into().unwrap();
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value.try_into().unwrap();
                                let addr_u: u8 = value2.try_into().unwrap();
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16).try_into().unwrap();
                            }
                            let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(val as u8);
                            ret = ret & 0x7F; // bit7, clear
                            self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                            self.write(self.cpu_pc.pc, T::from(ret as u8));
                        }
                    }
                }
            }
            OpcodeType::ROL => {
                match *_addressing.addr_mode {
                    AddrMode::ACC => {
                        println!("{}",format!("[DEBUG]: ROL ${}",dbg_str));
                        let a: T = self.get_register(CPUReg::A);
                        let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(a.try_into().unwrap());
                        if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                            ret = ret | BIN_BIT_0; // bit0 = C Flag Set
                        }else{
                            ret = ret & 0xFE; // bit0 = C Flag Clear
                        }
                        self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                        self.set_register(CPUReg::A, ret.try_into().unwrap());
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value.try_into().unwrap();
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16).try_into().unwrap();
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value.try_into().unwrap();
                                let addr_u: u8 = value2.try_into().unwrap();
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16).try_into().unwrap();
                            }

                            let mut ret: u8 = self.cpu_p_reg.c_flg_update_l_shit(val as u8);
                            if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                                ret = ret | BIN_BIT_0; // bit0 = C Flag Set
                            }else{
                                ret = ret & 0xFE; // bit0 = C Flag Clear
                            }
                            self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                            self.write(self.cpu_pc.pc, T::from(ret as u8));
                        }
                    }
                }
            }
            OpcodeType::ROR => {
                println!("{}",format!("[DEBUG]: ROR ${}",dbg_str));
                match *_addressing.addr_mode {
                    AddrMode::ACC => {
                        let a: T = self.get_register(CPUReg::A);
                        let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(a.try_into().unwrap());
                        if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                            ret = ret | BIN_BIT_7; // bit7 = C Flag Set
                        }else{
                            ret = ret & 0x7F;      // bit7 = C Flag Clear
                        }
                        self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                        self.set_register(CPUReg::A, ret.try_into().unwrap());
                    },
                    _ => {
                        if let Some(value) = operand {
                            let addr: u8 = value.try_into().unwrap();
                            let mut val: u8 = 0;
                            val =  self.read(addr as u16).try_into().unwrap();
                            if let Some(value2) = operand_second {
                                let addr_l: u8 = value.try_into().unwrap();
                                let addr_u: u8 = value2.try_into().unwrap();
                                let addr: u16 = (addr_u as u16) << 8 | addr_l as u16;
                                val = self.read(addr as u16).try_into().unwrap();
                            }

                            let mut ret: u8 = self.cpu_p_reg.c_flg_update_r_shit(val as u8);
                            if self.cpu_p_reg.get_status_flg(CARRY_FLG) {
                                ret = ret | BIN_BIT_7; // bit7 = C Flag Set
                            }else{
                                ret = ret & 0x7F;      // bit7 = C Flag Clear
                            }
                            self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                            self.write(self.cpu_pc.pc, T::from(ret as u8));
                        }
                    }
                }
            }

            // Load/Store Operations
            OpcodeType::LDA => {
                println!("{}",format!("[DEBUG]: LDA ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    ret = val.try_into().unwrap();

                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u8;
                    }
                }
                self.set_register(CPUReg::A, ret.try_into().unwrap());
            }
            OpcodeType::LDX => {
                println!("{}",format!("[DEBUG]: LDX ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    ret = val.try_into().unwrap();

                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u8;
                    }
                }
                self.set_register(CPUReg::X, ret.try_into().unwrap());
            }
            OpcodeType::LDY => {
                println!("{}",format!("[DEBUG]: LDY ${}",dbg_str));
                let mut ret: u8 = 0;
                if let Some(value) = operand {
                    let val: u16 = value.try_into().unwrap();
                    ret = val.try_into().unwrap();

                    if let Some(value2) = operand_second {
                        let val2: u16 = value2.try_into().unwrap();
                        ret = ((val2 << 8) | val) as u8;
                    }
                }
                self.set_register(CPUReg::Y, ret.try_into().unwrap());
            }
            OpcodeType::STA => {
                println!("{}",format!("[DEBUG]: STA ${}",dbg_str));
                let a: T = self.get_register(CPUReg::A);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = val as u16;

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, a);
            }
            OpcodeType::STX => {
                println!("{}",format!("[DEBUG]: STX ${}",dbg_str));
                let x: T = self.get_register(CPUReg::X);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = val as u16;

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, x);
            }
            OpcodeType::STY => {
                println!("{}",format!("[DEBUG]: STY ${}",dbg_str));
                let y: T = self.get_register(CPUReg::Y);
                let mut addr: u16 = 0;

                if let Some(value) = operand {
                    let val: u8 = value.try_into().unwrap();
                    addr = val as u16;

                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        addr = (val2 as u16) << 8 | val as u16;
                    }
                }
                self.write(addr, y);
            }

            // Register Transfer Operations/レジスタ転送関連の命令
            OpcodeType::TAX => {
                println!("{}",format!("[DEBUG]: TAX ${}",dbg_str));
                let a = self.get_register(CPUReg::A);
                self.set_register(CPUReg::X, a);
            }
            OpcodeType::TAY => {
                println!("{}",format!("[DEBUG]: TAY ${}",dbg_str));
                let a = self.get_register(CPUReg::A);
                self.set_register(CPUReg::Y, a);
            }
            OpcodeType::TXA => {
                println!("{}",format!("[DEBUG]: TXA ${}",dbg_str));
                let x = self.get_register(CPUReg::X);
                self.set_register(CPUReg::A, x);
            }
            OpcodeType::TYA => {
                println!("{}",format!("[DEBUG]: TYA ${}",dbg_str));
                let y = self.get_register(CPUReg::Y);
                self.set_register(CPUReg::A, y);
            }

            // Stack Operations / スタック関連の命令
            OpcodeType::TSX => {
                println!("{}",format!("[DEBUG]: TSX ${}",dbg_str));
                let sp = self.get_register(CPUReg::SP);
                self.set_register(CPUReg::X, sp);
            }
            OpcodeType::TXS => {
                println!("{}",format!("[DEBUG]: TXS ${}",dbg_str));
                let x = self.get_register(CPUReg::X);
                self.set_register(CPUReg::SP, x);
            }
            OpcodeType::PHA => {
                println!("{}",format!("[DEBUG]: PHA ${}",dbg_str));
                let a = self.get_register(CPUReg::A);
                self.push_stack(a);
            }
            OpcodeType::PHP => {
                println!("{}",format!("[DEBUG]: PHP ${}",dbg_str));
                let p = self.cpu_p_reg.get_status_flg_all();
                self.push_stack(p.try_into().unwrap());
            }
            OpcodeType::PLA => {
                println!("{}",format!("[DEBUG]: PLA ${}",dbg_str));
                let value = self.pop_stack();
                self.set_register(CPUReg::A, value);
                self.cpu_p_reg.nzv_flg_update(value.try_into().unwrap());
            }
            OpcodeType::PLP => {
                println!("{}",format!("[DEBUG]: PLP ${}",dbg_str));
                let value = self.pop_stack();
                self.cpu_p_reg.set_status_flg_all(value.try_into().unwrap());
            }

            // Status Flag Operations / ステータスフラグ関連の命令
            OpcodeType::BIT => {
                println!("{}",format!("[DEBUG]: BIT ${}",dbg_str));
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
            }
            OpcodeType::CLC => {
                self.cpu_p_reg.cls_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: CLC ${}",dbg_str));
            }
            OpcodeType::CLD => {
                self.cpu_p_reg.cls_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: CLD ${}",dbg_str));
            }
            OpcodeType::CLI => {
                self.cpu_p_reg.cls_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: CLI ${}",dbg_str));
            }
            OpcodeType::CLV => {
                self.cpu_p_reg.cls_status_flg(OVERFLOW_FLG);
                println!("{}",format!("[DEBUG]: CLV ${}",dbg_str));
            }
            OpcodeType::SEC => {
                self.cpu_p_reg.set_status_flg(CARRY_FLG);
                println!("{}",format!("[DEBUG]: SEC ${}",dbg_str));
            }
            OpcodeType::SED => {
                self.cpu_p_reg.set_status_flg(DECIMAL_MODE_FLG);
                println!("{}",format!("[DEBUG]: SED ${}",dbg_str));
            }
            OpcodeType::SEI => {
                self.cpu_p_reg.set_status_flg(INTERRUPT_DISABLE_FLG);
                println!("{}",format!("[DEBUG]: SEI ${}",dbg_str));
            }

            // Branch Operations / 分岐命令
            OpcodeType::BCC => {
                println!("{}",format!("[DEBUG]: BCC ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(CARRY_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BCS => {
                println!("{}",format!("[DEBUG]: BCS ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(CARRY_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BEQ => {
                println!("{}",format!("[DEBUG]: BEQ ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(ZERO_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BNE => {
                println!("{}",format!("[DEBUG]: BNE ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(ZERO_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BVC => {
                println!("{}",format!("[DEBUG]: BVC ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(OVERFLOW_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BVS => {
                println!("{}",format!("[DEBUG]: BVS ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(OVERFLOW_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BPL => {
                println!("{}",format!("[DEBUG]: BPL ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(NEGATIVE_FLG) != true {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }
            OpcodeType::BMI => {
                println!("{}",format!("[DEBUG]: BMI ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(NEGATIVE_FLG) != false {
                    if let Some(value) = operand {
                        if let Some(value2) = operand_second {
                            let val: u16 = value.try_into().unwrap();
                            let val2:u16 = value2.try_into().unwrap();
                            let branch_addr: u16 = (val2 << 8) | val;
                            self.cpu_pc.pc = branch_addr;
                            jmp_flg = true;
                        }
                    }
                }
            }

            // Jump and Call Operations
            OpcodeType::JMP => {
                println!("{}",format!("[DEBUG]: JMP ${}",dbg_str));
                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        let jump_addr: u16 = (val2 as u16) << 8 | val as u16;
                        self.cpu_pc.pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            OpcodeType::JSR => {
                println!("{}",format!("[DEBUG]: JSR {}",dbg_str));
                let mut jump_addr: u16 = 0x00;
                let return_addr: u16 = self.cpu_pc.pc;
                // let return_addr: u16 = self.cpu_pc.pc + 1;
                self.push_stack((return_addr & 0x00FF).try_into().unwrap());
                self.push_stack(((return_addr & 0xFF00) >> 0x0008).try_into().unwrap());

                if let Some(value) = operand {
                    if let Some(value2) = operand_second {
                        let val: u8 = value.try_into().unwrap();
                        let val2: u8 = value2.try_into().unwrap();
                        jump_addr = (val2 as u16) << 8 | val as u16;
                        self.cpu_pc.pc = jump_addr;
                        jmp_flg = true;
                    }
                }
            }
            OpcodeType::RTS => {
                println!("{}",format!("[DEBUG]: RTS ${}",dbg_str));
                let return_addr_u: u16 = self.pop_stack().try_into().unwrap();
                let return_addr_l: u16 = self.pop_stack().try_into().unwrap();
                let return_addr: u16 = (return_addr_u << 8) | return_addr_l;
                self.cpu_pc.pc = return_addr + 1;
                jmp_flg = true;
            }

            // Intrrupt Operations / 割込み関連
            OpcodeType::RTI => {
                println!("{}",format!("[DEBUG]: RTI ${}",dbg_str));
                let status = self.pop_stack();
                self.cpu_p_reg.set_status_flg_all(status.into());
                let return_addr_l: u16 = self.pop_stack().try_into().unwrap();
                let return_addr_u: u16 = self.pop_stack().try_into().unwrap();
                let return_addr: u16 = (return_addr_u << 8) | return_addr_l;
                self.cpu_pc.pc = return_addr;
                jmp_flg = true;
            }
            OpcodeType::BRK => {
                println!("{}",format!("[DEBUG]: BRK ${}",dbg_str));
                if self.cpu_p_reg.get_status_flg(BREAK_COMMAND_FLG) != true {
                    self.cpu_pc.pc += 1;
                    self.cpu_p_reg.set_status_flg(BREAK_COMMAND_FLG);
                    self.push_stack((self.cpu_pc.pc & 0x00FF).try_into().unwrap());
                    self.push_stack(((self.cpu_pc.pc & 0xFF00) >> 0x0008).try_into().unwrap());
                    self.push_stack(self.cpu_p_reg.get_status_flg_all().try_into().unwrap());
                    self.cpu_p_reg.set_status_flg(BREAK_COMMAND_FLG);
                    let mut _jmp_addr: T = self.read(ADDR_VEC_TBL_IRQ);
                    _jmp_addr = self.read(ADDR_VEC_TBL_IRQ + 1) << 0x0008;
                    self.cpu_pc.pc = _jmp_addr.try_into().unwrap();
                    jmp_flg = true;
                    print!("BRK Jmp to: ${:04X}", self.cpu_pc.pc);
                }
            }

            // Other
            OpcodeType::STP | _ => {
                // TODO STPと未定義命令をどうするか
                println!("[DEBUG]: Undefined Instruction!");
            }
        }

        // pc ++
        if (operand != None) & (jmp_flg != true)
        {
            self.cpu_pc.pc += 1;
        }
    }

fn push_stack(&mut self, data: T) {
    let sp = self.get_register(CPUReg::SP);
    let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
    self.write(address, data);
    self.set_register(CPUReg::SP, sp - T::from(1u8));
}

fn pop_stack(&mut self) -> T {
    let sp = self.get_register(CPUReg::SP) + T::from(1u8);
    self.set_register(CPUReg::SP, sp);
    let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
    self.read(address)
}

    fn read_operand(&mut self, addressing: Addressing) -> (Option<T>, Option<T>, String)
    {
        self.cpu_pc.pc += 1;
        let oprand:u8 = self.read(self.cpu_pc.pc).try_into().unwrap();

        match *addressing.addr_mode {
            AddrMode::ACC => {
                let acc:u8 = self.get_register(CPUReg::A).try_into().unwrap();
                (Some(self.get_register(CPUReg::A)),
                None,
                format!("{:#02X} (ACC)", acc))
            }
            AddrMode::IMM => {
                (Some(self.read(self.cpu_pc.pc)),
                None,
                format!("{:#02X} (IMM)",oprand))
            }
            AddrMode::ZPG => {
                (Some(self.read(self.cpu_pc.pc)),
                None,
                format!("{:#02X} (ZPG)",oprand))
            }
            AddrMode::ZpgX => {
                let address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                (Some(self.read(address.try_into().unwrap())),
                None,
                format!("{:#02X} (ZpgX)",oprand))
            }
            AddrMode::ZpgY => {
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                (Some(self.read(address.try_into().unwrap())),
                None,
                format!("{:#02X} (ZpgY)",oprand))
            }
            AddrMode::ABS => {
                let addr_l:u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                self.cpu_pc.pc += 1;
                let addr_u:u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                (Some(T::from(addr_l as u8)),
                Some(T::from(addr_u as u8)),
                format!("{:#02X} {:#02X} (ABS)",addr_l, addr_u))
            }
            AddrMode::AbsX => {
                let mut addr_l: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                addr_l |= TryInto::<u16>::try_into(self.get_register(CPUReg::X)).unwrap();
                self.cpu_pc.pc += 1;
                let mut addr_u: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                addr_u |= addr_l;
                (Some(T::from(addr_l as u8)),
                Some(T::from(addr_u as u8)),
                format!("{:#02X} {:#02X} (AbsX)",addr_l, addr_u))
            }
            AddrMode::AbsY => {
                let mut addr_l: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                addr_l |= TryInto::<u16>::try_into(self.get_register(CPUReg::Y)).unwrap();
                self.cpu_pc.pc += 1;
                let mut addr_u: u16 = self.read(self.cpu_pc.pc).try_into().unwrap();
                addr_u |= addr_l;
                (Some(T::from(addr_l as u8)),
                Some(T::from(addr_u as u8)),
                format!("{:#02X} {:#02X} (AbsY)",addr_l, addr_u))
            }
            AddrMode::IND => {
                let address: T = self.read(self.cpu_pc.pc);
                (Some(self.read(address.try_into().unwrap())),
                None,
                format!("{:#02X} (IND)",oprand))
            }
            AddrMode::IndX => {
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                let address: T = self.read(base_address.try_into().unwrap());
                (Some(self.read(address.try_into().unwrap())),
                None,
                format!("{:#02X} (IndX)",oprand))
            }
            AddrMode::IndY => {
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                let address: T = self.read(base_address.try_into().unwrap());
                (Some(self.read(address.try_into().unwrap())),
                None,
                format!("{:#02X} (IndY)",oprand))
            }
            AddrMode::REL => { // Relative Addressing(相対アドレッシング)
                let offset: u8 = self.read(self.cpu_pc.pc + 2).try_into().unwrap();
                let s_offset: i8 = offset as i8;
                let addr: u16 = (self.cpu_pc.pc as i16).wrapping_add(s_offset as i16) as u16;
                let addr_l: u8 = addr as u8;
                let addr_u: u8 = (addr >> 8) as u8;
                (Some(T::from(addr_l as u8)),
                Some(T::from(addr_u as u8)),
                format!("${:04X} (REL)(Offset: 0x{:02X}({}))", addr, s_offset, s_offset))
            }
            AddrMode::IMPL => { // Implied Addressing
                // Not, Have Operand
                (None, None,format!("(IMPL)"))
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
    println!("[DEBUG]: A:0x{:02X},X:0x{:02X},Y:0x{:02X},S:0x{:02X},P:{:08b},PC:0x{:04X}",a,x,y,sp,p,pc);
}

fn cpu_proc(cpu :&mut RP2A03<u8>)
{
    let op_code = cpu.fetch_instruction();
    let (opcode, addressing) = cpu.decode_instruction(op_code);
    cpu.execute_instruction(opcode, addressing);
}

pub fn cpu_stop(flg: bool) {
    if *S_CPU_STOP {
        println!("[DEBUG]: CPU Stop");
    }
}

pub fn cpu_reset() -> Option<RP2A03<u8>> {
    S_CPU.call_once(|| {
        let mut cpu_data = S_CPU_DATA.lock().unwrap();
        *cpu_data = Some(RP2A03 {
            cpu_reg: [0u8; 4],
            cpu_p_reg: StatusRegister::new(),
            cpu_pc: ProgramCounter::new(),
            nes_mem: NESMemory::new(),
            interrupt: Interrupt::new(),
        });
    });

    if let Some(cpu) = S_CPU_DATA.lock().unwrap().as_mut() {
        cpu.nes_mem.mem_reset();
        cpu.reset();
    }

    S_CPU_DATA.lock().unwrap().clone()
}

pub fn cpu_main() {
    if *S_CPU_STOP {
        return;
    }

    unsafe {
        S_CPU_DATA.lock().unwrap().as_mut().map(|cpu| {
            cpu_reg_show(cpu);
            cpu_proc(cpu);
        });
    }
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