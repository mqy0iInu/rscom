use std::thread;
use std::time::Duration;
use std::convert::TryInto;
use std::convert::From;
use std::num::Wrapping;
use std::rc::Rc;

pub const BIN_BIT_7: u8 = 0x80; // bit7
pub const BIN_BIT_6: u8 = 0x40; // bit6
pub const BIN_BIT_5: u8 = 0x20; // bit5
pub const BIN_BIT_4: u8 = 0x10; // bit4
pub const BIN_BIT_3: u8 = 0x08; // bit3
pub const BIN_BIT_2: u8 = 0x04; // bit2
pub const BIN_BIT_1: u8 = 0x02; // bit1
pub const BIN_BIT_0: u8 = 0x01; // bit0

enum CPUReg {
    A,   // 汎用レジスタ（アキュムレータ）... 演算の結果やデータを一時的に保持する。
    X,   // インデックスレジスタX         ... ループや配列のインデックスなどに使用する。
    Y,   // インデックスレジスタY         ... ループや配列のインデックスなどに使用する。
    SP,  // スタックポインタ              ... スタックのトップアドレスを示す。
}

enum OpcodeType {
    // Load/Store Operations
    LDA, // Load Accumulator / アキュムレータをロードする
    LDX, // Load X Register / X レジスタをロードする
    LDY, // Load Y Register / Y レジスタをロードする
    STA, // Store Accumulator / アキュムレータをストアする
    STX, // Store X Register / X レジスタをストアする
    STY, // Store Y Register / Y レジスタをストアする

    // Register Transfer Operations
    TAX, // Transfer Accumulator to X / アキュムレータを X に転送する
    TAY, // Transfer Accumulator to Y / アキュムレータを Y に転送する
    TXA, // Transfer X to Accumulator / X をアキュムレータに転送する
    TYA, // Transfer Y to Accumulator / Y をアキュムレータに転送する

    // Stack Operations
    TSX, // Transfer Stack Pointer to X / スタックポインタを X に転送する
    TXS, // Transfer X to Stack Pointer / X をスタックポインタに転送する
    PHA, // Push Accumulator / アキュムレータをプッシュする
    PHP, // Push Processor Status / プロセッサステータスをプッシュする
    PLA, // Pull Accumulator / アキュムレータをプルする
    PLP, // Pull Processor Status / プロセッサステータスをプルする

    // Logical Operations
    AND, // Logical AND / 論理積を計算する
    ORA, // Logical OR / 論理和を計算する
    EOR, // Exclusive OR / 排他的論理和を計算する
    BIT, // Bit Test / ビットテストを実行する

    // Arithmetic Operations
    ADC, // Add with Carry / キャリーフラグを考慮して加算する
    SBC, // Subtract with Carry / キャリーフラグを考慮して減算する
    CMP, // Compare Accumulator / アキュムレータを比較する
    CPX, // Compare X Register / X レジスタを比較する
    CPY, // Compare Y Register / Y レジスタを比較する
    INC, // Increment Memory / メモリをインクリメントする
    INX, // Increment X Register / X レジスタをインクリメントする
    INY, // Increment Y Register / Y レジスタをインクリメントする
    DEC, // Decrement Memory / メモリをデクリメントする
    DEX, // Decrement X Register / X レジスタをデクリメントする
    DEY, // Decrement Y Register / Y レジスタをデクリメントする

    // Shift and Rotate Operations
    ASL, // Arithmetic Shift Left / 算術左シフトを実行する
    LSR, // Logical Shift Right / 論理右シフトを実行する
    ROL, // Rotate Left / 左にローテートする
    ROR, // Rotate Right / 右にローテートする

    // Jump and Call Operations
    JMP, // Jump / ジャンプする
    JSR, // Jump to Subroutine / サブルーチンにジャンプする

    // Branch Operations
    BCC, // Branch if Carry Clear / キャリーフラグがクリアされている場合に分岐する
    BCS, // Branch if Carry Set / キャリーフラグがセットされている場合に分岐する
    BNE, // Branch if Not Equal / ゼロフラグがクリアされている場合に分岐する
    BEQ, // Branch if Equal / ゼロフラグがセットされている場合に分岐する
    BPL, // Branch if Positive / ネガティブフラグがクリアされている場合に分岐する
    BMI, // Branch if Minus / ネガティブフラグがセットされている場合に分岐する
    BVC, // Branch if Overflow Clear / オーバーフローフラグがクリアされている場合に分岐する
    BVS, // Branch if Overflow Set / オーバーフローフラグがセットされている場合に分岐する

    // Status Flag Operations
    CLC, // Clear Carry Flag / キャリーフラグをクリアする
    CLD, // Clear Decimal Mode / デシマルモードをクリアする
    CLI, // Clear Interrupt Disable / 割り込み禁止フラグをクリアする
    CLV, // Clear Overflow Flag / オーバーフローフラグをクリアする
    SEC, // Set Carry Flag / キャリーフラグをセットする
    SED, // Set Decimal Mode / デシマルモードをセットする
    SEI, // Set Interrupt Disable / 割り込み禁止フラグをセットする

    // Intrrupt Operations
    RTS, // Return from Subroutine / サブルーチンからリターンする
    RTI, // Return from Interrupt / 割り込みからリターンす
    BRK, // Break / プログラムの実行を一時停止し、割り込み処理に制御を移す命令

    // Other
    NOP, // No Operation / オペレーションなし
    UNK, // Unknown Instruction / 不明な命令
    STP, // Stop / 停止
}

enum AddrMode {
    Accumulator,   // アキュムレータ (Accumulator) - 命令のオペランドはアキュムレータレジスタ自体を使用する。
    Immediate,     // イミディエイト (Immediate) - 命令のオペランドは即値データそのもの。
    Absolute,      // アブソリュート (Absolute) - 命令のオペランドはメモリ上の絶対アドレス。
    ZeroPage,      // ゼロページ (Zero Page) - 命令のオペランドはゼロページアドレス。上位バイトは常にゼロ。
    ZeroPageX,     // ゼロページ、Xインデックス (Zero Page, X-Indexed) - 命令のオペランドはゼロページアドレスとXレジスタの値の和。
    ZeroPageY,     // ゼロページ、Yインデックス (Zero Page, Y-Indexed) - 命令のオペランドはゼロページアドレスとYレジスタの値の和。
    AbsoluteX,     // アブソリュート、Xインデックス (Absolute, X-Indexed) - 命令のオペランドは絶対アドレスとXレジスタの値の和。
    AbsoluteY,     // アブソリュート、Yインデックス (Absolute, Y-Indexed) - 命令のオペランドは絶対アドレスとYレジスタの値の和。
    Indirect,      // インダイレクト (Indirect) - 命令のオペランドはジャンプ先の絶対アドレスを格納しているアドレス。
    IndirectX,     // インデックスインダイレクト、Xインデックス (Indexed Indirect, X-Indexed) - 命令のオペランドはXレジスタの値を加えたアドレスから間接的にジャンプする。
    IndirectY,     // インダイレクトインデックス、Yインデックス (Indirect Indexed, Y-Indexed) - 命令のオペランドはYレジスタの値を加えたアドレスから間接的にジャンプする。
    Relative,      // リラティブ (Relative) - 命令のオペランドは相対的なジャンプオフセット。オペランドはプログラムカウンタの値に加算される。
    Implied,       // インプライド (Implied) - 命令のオペランドは存在しない。※一部の命令は暗黙的に特定のレジスタやフラグを操作する。
}

struct Opcode {
    opcode_type: OpcodeType,
    // Add other fields here
}

#[derive(Clone)]
struct Addressing {
    addr_mode: Rc<AddrMode>,
    // Add other fields here
}

trait CPU<T> {
    fn reset(&mut self);
    fn read(&mut self, address: u16) -> T;
    fn write(&mut self, address: u16, data: T);
    fn get_register(&self, register: CPUReg) -> T;
    fn set_register(&mut self, register: CPUReg, value: T);
    fn fetch_instruction(&mut self) -> T;
    fn decode_instruction(&mut self, op_code: T) -> (Opcode, Addressing);
    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing);
    fn push_stack(&mut self, data: T);
    fn pop_stack(&mut self) -> T;
    fn read_operand(&mut self, addressing: Addressing) -> Option<T>;
}

struct ProgramCounter {
    pc: u16,
}

impl ProgramCounter {
    pub const ADDR_CHR_ROM: u16 = 0x4020;     // CHR-ROM TOP
    pub const ADDR_PRG_RAM: u16 = 0xFFFE;     // PRG-RAM TOP
    pub const ADDR_PRG_ROM: u16 = 0x8000;     // PRG-ROM TOP

    pub const ADDR_VEC_TBL_RST: u16 = 0xFFFC; // RESET Vector Table
    pub const ADDR_VEC_TBL_IRQ: u16 = 0xFFFE; // IRQ Vector Table
    pub const ADDR_VEC_TBL_NMI: u16 = 0xFFFA; // NMI Vector Table

    fn new() -> Self {
        ProgramCounter {
            // TODO PCの初期位置
            // pc : Self::ADDR_VEC_TBL_RST,
            pc : Self::ADDR_PRG_ROM,

             // リセットベクタ
            // pc : Self::ADDR_VEC_TBL_RST,
        }
    }
}

/// RP2A03のステータスレジスタ
struct StatusRegister {
    p_reg: u8,
}

impl StatusRegister {
    pub const NEGATIVE_FLG: u8 = 0b1000_0000;              // bit7: N Flag. ネガティブフラグ。演算の結果が負の場合にセットされる。
    pub const OVERFLOW_FLG: u8 = 0b0100_0000;              // bit6: V Flag. オーバーフローフラグ。符号付き演算の結果がオーバーフローした場合にセットされる。
    pub const R_FLG: u8 = 0b0010_0000;                     // bit5: R Flag. Reaerved.予約済 (常に1固定)
    pub const BREAK_COMMAND_FLG: u8 = 0b0001_0000;         // bit4: B Flag. ブレークコマンドフラグ。BRK命令が実行されたときにセットされる。
    pub const DECIMAL_MODE_FLG: u8 = 0b0000_1000;          // bit3: D Flag. 10進モードフラグ。BCD（Binary-Coded Decimal）演算のためのアドレッシングモードを制御する。
    pub const INTERRUPT_DISABLE_FLG: u8 = 0b0000_0100;     // bit2: I Flag. 割り込み無効フラグ (0 ... IRQ許可, 1 ... IRQをマスク)
    pub const ZERO_FLG: u8 = 0b0000_0010;                  // bit1: Z Flag. ゼロフラグ。演算の結果がゼロの場合にセットされる。
    pub const CARRY_FLG: u8 = 0b0000_0001;                 // bit0: C Flag. キャリーフラグ。算術演算でのキャリーや借りがある場合にセットされる。

    fn new() -> Self {
        StatusRegister {
            p_reg: Self::R_FLG, // ビット5: Reaerved.予約済 (常に1固定)
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

    fn cls_status_flg_all(&mut self) {
        self.p_reg = Self::R_FLG;
    }

    fn nzv_flg_update(&mut self, val: u8) {
        if val == 0{
            self.set_status_flg(Self::ZERO_FLG);
        }else{
            self.cls_status_flg(Self::ZERO_FLG);
        }

        if (val & BIN_BIT_7) != 0 {
            self.set_status_flg(Self::NEGATIVE_FLG);
        }else{
            self.cls_status_flg(Self::NEGATIVE_FLG);
        }
    }

    fn c_flg_update_add(&mut self, val_a: u8,  val_b: u8) -> u8{
        let mut ret: u16 = val_a as u16;
        ret += val_b as u16;
        if ret >  0x00FF {
            self.set_status_flg(Self::CARRY_FLG);
            0x00
        }else{
            self.cls_status_flg(Self::CARRY_FLG);
            ret as u8
        }
    }

    fn c_flg_update_l_shit(&mut self, val: u8) -> u8{
        let mut ret: u16 = val as u16;

        if (val & BIN_BIT_7) != 0 {
            self.set_status_flg(Self::CARRY_FLG);
        }else {
            self.cls_status_flg(Self::CARRY_FLG);
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
            self.set_status_flg(Self::CARRY_FLG);
        }else {
            self.cls_status_flg(Self::CARRY_FLG);
        }

        ret = ret >> 1;
        if ret <= 0x00 {
            ret = 0;
        }
        ret as u8
    }
}


struct NESMemory {
    wram: [u8; 2048],         // WRAM ... 2KB (For RP2A03)
    vram: [u8; 2048],         // VRAM ... 2KB (For PPU)
    ppu_registers: [u8; 8],   // PPUレジスタ
    apu_registers: [u8; 24],  // APUレジスタ

    ext_rom: Vec<u8>,         // CHR ROM ... 8KB or 16KB
    prg_ram: Vec<u8>,         // PRG RAM
    prg_rom: Vec<u8>,         // PRG ROM ... 8KB ~ 1MB
}

impl NESMemory {
    fn new() -> Self {
        NESMemory {
            wram: [0; 2048],
            vram: [0; 2048],
            ppu_registers: [0; 8],
            apu_registers: [0; 24],

            ext_rom: Vec::new(),
            prg_ram: Vec::new(),
            prg_rom: Vec::new(),
        }
    }

    fn mem_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x07FF => self.wram[address as usize],                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(address % 0x0800) as usize],          // RAMのミラーリング
            0x2000..=0x2007 => self.ppu_registers[(address - 0x2000) as usize], // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(address - 0x2000) as usize],          // VRAM ... 2KB (For PPU)
            0x4000..=0x4017 => self.apu_registers[(address - 0x4000) as usize], // APUレジスタ
            0x4020..=0x5FFF => self.ext_rom[(address - 0x4020) as usize],       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize],       // PRG RAM
            0x8000..=0xFFFF => self.prg_rom[(address - 0x8000) as usize],       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid memory address: {:#06x}", address),
        }
    }

    fn mem_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x07FF => self.wram[address as usize] = data,                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(address % 0x0800) as usize] = data,          // RAMのミラーリング
            0x2000..=0x2007 => self.ppu_registers[(address - 0x2000) as usize] = data, // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(address - 0x2000) as usize] = data,          // VRAM ... 2KB (For PPU)
            0x4000..=0x4017 => self.apu_registers[(address - 0x4000) as usize] = data, // APUレジスタ
            0x4020..=0x5FFF => self.ext_rom[(address - 0x4020) as usize] = data,       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.prg_ram[(address - 0x6000) as usize] = data,       // PRG RAM
            0x8000..=0xFFFF => self.prg_rom[(address - 0x8000) as usize] = data,       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid memory address: {:#06x}", address),
        }
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
    T: Copy + From<u8> + Into<u8> + std::ops::Add<Output = T> + std::ops::Sub<Output = T>
        + std::ops::BitAnd<Output = T> + std::ops::BitOr<Output = T>+ std::ops::BitXor<Output = T>
        + TryFrom<u16> + Into<u16> + Into<i32> + PartialEq + PartialOrd + std::ops::Shl<u8, Output = T>,
    <T as std::convert::TryFrom<u16>>::Error: std::fmt::Debug,i32: From<T>,
{
    fn reset(&mut self){
        // Implement reset routine
        self.set_register(CPUReg::A, T::from(0u8));
        self.set_register(CPUReg::X, T::from(0u8));
        self.set_register(CPUReg::Y, T::from(0u8));
        self.set_register(CPUReg::SP, T::from(0x00u8));
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
            0x00 => { opcode_type = OpcodeType::BRK; addr_mode = Rc::new(AddrMode::Implied); },
            0x01 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::IndirectX); },
            0x05 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x06 => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x08 => { opcode_type = OpcodeType::PHP; addr_mode = Rc::new(AddrMode::Implied); },
            0x09 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::Immediate); },
            0x0A => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::Accumulator); },
            0x0D => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::Absolute); },
            0x0E => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::Absolute); },
            0x10 => { opcode_type = OpcodeType::BPL; addr_mode = Rc::new(AddrMode::Relative); },
            0x11 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::IndirectY); },
            0x15 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x16 => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x18 => { opcode_type = OpcodeType::CLC; addr_mode = Rc::new(AddrMode::Implied); },
            0x19 => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0x1D => { opcode_type = OpcodeType::ORA; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x1E => { opcode_type = OpcodeType::ASL; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x20 => { opcode_type = OpcodeType::JSR; addr_mode = Rc::new(AddrMode::Absolute); },
            0x21 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::IndirectX); },
            0x24 => { opcode_type = OpcodeType::BIT; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x25 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x26 => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x28 => { opcode_type = OpcodeType::PLP; addr_mode = Rc::new(AddrMode::Implied); },
            0x29 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::Immediate); },
            0x2A => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::Accumulator); },
            0x2C => { opcode_type = OpcodeType::BIT; addr_mode = Rc::new(AddrMode::Absolute); },
            0x2D => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::Absolute); },
            0x2E => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::Absolute); },
            0x30 => { opcode_type = OpcodeType::BMI; addr_mode = Rc::new(AddrMode::Relative); },
            0x31 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::IndirectY); },
            0x35 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x36 => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x38 => { opcode_type = OpcodeType::SEC; addr_mode = Rc::new(AddrMode::Implied); },
            0x39 => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0x3D => { opcode_type = OpcodeType::AND; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x3E => { opcode_type = OpcodeType::ROL; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x40 => { opcode_type = OpcodeType::RTI; addr_mode = Rc::new(AddrMode::Implied); },
            0x41 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::IndirectX); },
            0x45 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x46 => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x48 => { opcode_type = OpcodeType::PHA; addr_mode = Rc::new(AddrMode::Implied); },
            0x49 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::Immediate); },
            0x4A => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::Accumulator); },
            0x4C => { opcode_type = OpcodeType::JMP; addr_mode = Rc::new(AddrMode::Absolute); },
            0x4D => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::Absolute); },
            0x4E => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::Absolute); },
            0x50 => { opcode_type = OpcodeType::BVC; addr_mode = Rc::new(AddrMode::Relative); },
            0x51 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::IndirectY); },
            0x55 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x56 => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x58 => { opcode_type = OpcodeType::CLI; addr_mode = Rc::new(AddrMode::Implied); },
            0x59 => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0x5D => { opcode_type = OpcodeType::EOR; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x5E => { opcode_type = OpcodeType::LSR; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x60 => { opcode_type = OpcodeType::RTS; addr_mode = Rc::new(AddrMode::Implied); },
            0x61 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::IndirectX); },
            0x65 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x66 => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x68 => { opcode_type = OpcodeType::PLA; addr_mode = Rc::new(AddrMode::Implied); },
            0x69 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::Immediate); },
            0x6A => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::Accumulator); },
            0x6C => { opcode_type = OpcodeType::JMP; addr_mode = Rc::new(AddrMode::Indirect); },
            0x6D => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::Absolute); },
            0x6E => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::Absolute); },
            0x70 => { opcode_type = OpcodeType::BVS; addr_mode = Rc::new(AddrMode::Relative); },
            0x71 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::IndirectY); },
            0x75 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x76 => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x78 => { opcode_type = OpcodeType::SEI; addr_mode = Rc::new(AddrMode::Implied); },
            0x79 => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0x7D => { opcode_type = OpcodeType::ADC; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x7E => { opcode_type = OpcodeType::ROR; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0x81 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::IndirectX); },
            0x84 => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x85 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x86 => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x88 => { opcode_type = OpcodeType::DEY; addr_mode = Rc::new(AddrMode::Implied); },
            0x8A => { opcode_type = OpcodeType::TXA; addr_mode = Rc::new(AddrMode::Implied); },
            0x8C => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::Absolute); },
            0x8D => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::Absolute); },
            0x8E => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::Absolute); },
            0x90 => { opcode_type = OpcodeType::BCC; addr_mode = Rc::new(AddrMode::Relative); },
            0x91 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::IndirectY); },
            0x94 => { opcode_type = OpcodeType::STY; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x95 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x96 => { opcode_type = OpcodeType::STX; addr_mode = Rc::new(AddrMode::ZeroPageY); },
            0x98 => { opcode_type = OpcodeType::TYA; addr_mode = Rc::new(AddrMode::Implied); },
            0x99 => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0x9A => { opcode_type = OpcodeType::TXS; addr_mode = Rc::new(AddrMode::Implied); },
            0x9D => { opcode_type = OpcodeType::STA; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xA0 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::Immediate); },
            0xA1 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::IndirectX); },
            0xA2 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::Immediate); },
            0xA4 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xA5 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xA6 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xA8 => { opcode_type = OpcodeType::TAY; addr_mode = Rc::new(AddrMode::Implied); },
            0xA9 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::Immediate); },
            0xAA => { opcode_type = OpcodeType::TAX; addr_mode = Rc::new(AddrMode::Implied); },
            0xAC => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::Absolute); },
            0xAD => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::Absolute); },
            0xAE => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::Absolute); },
            0xB0 => { opcode_type = OpcodeType::BCS; addr_mode = Rc::new(AddrMode::Relative); },
            0xB1 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::IndirectY); },
            0xB4 => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xB5 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xB6 => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::ZeroPageY); },
            0xB8 => { opcode_type = OpcodeType::CLV; addr_mode = Rc::new(AddrMode::Implied); },
            0xB9 => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0xBA => { opcode_type = OpcodeType::TSX; addr_mode = Rc::new(AddrMode::Implied); },
            0xBC => { opcode_type = OpcodeType::LDY; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xBD => { opcode_type = OpcodeType::LDA; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xBE => { opcode_type = OpcodeType::LDX; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0xC0 => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::Immediate); },
            0xC1 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::IndirectX); },
            0xC4 => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xC5 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xC6 => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xC8 => { opcode_type = OpcodeType::INY; addr_mode = Rc::new(AddrMode::Implied); },
            0xC9 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::Immediate); },
            0xCA => { opcode_type = OpcodeType::DEX; addr_mode = Rc::new(AddrMode::Implied); },
            0xCC => { opcode_type = OpcodeType::CPY; addr_mode = Rc::new(AddrMode::Absolute); },
            0xCD => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::Absolute); },
            0xCE => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::Absolute); },
            0xD0 => { opcode_type = OpcodeType::BNE; addr_mode = Rc::new(AddrMode::Relative); },
            0xD1 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::IndirectY); },
            0xD5 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xD6 => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xD8 => { opcode_type = OpcodeType::CLD; addr_mode = Rc::new(AddrMode::Implied); },
            0xD9 => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0xDD => { opcode_type = OpcodeType::CMP; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xDE => { opcode_type = OpcodeType::DEC; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xE0 => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::Immediate); },
            0xE1 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::IndirectX); },
            0xE4 => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xE5 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xE6 => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0xE8 => { opcode_type = OpcodeType::INX; addr_mode = Rc::new(AddrMode::Implied); },
            0xE9 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::Immediate); },
            0xEC => { opcode_type = OpcodeType::CPX; addr_mode = Rc::new(AddrMode::Absolute); },
            0xED => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::Absolute); },
            0xEE => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::Absolute); },
            0xF0 => { opcode_type = OpcodeType::BEQ; addr_mode = Rc::new(AddrMode::Relative); },
            0xF1 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::IndirectY); },
            0xF5 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xF6 => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0xF8 => { opcode_type = OpcodeType::SED; addr_mode = Rc::new(AddrMode::Implied); },
            0xF9 => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::AbsoluteY); },
            0xFD => { opcode_type = OpcodeType::SBC; addr_mode = Rc::new(AddrMode::AbsoluteX); },
            0xFE => { opcode_type = OpcodeType::INC; addr_mode = Rc::new(AddrMode::AbsoluteX); },

            // NOP
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::Implied); },
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::Immediate); },
            0x04 | 0x44 | 0x64 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::ZeroPage); },
            0x14 | 0x34 | 0x54 | 0x74| 0xD4| 0xF4 => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::ZeroPageX); },
            0x0C => { opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::Absolute); },
            0x1C | 0x3C | 0x5C | 0x7C| 0xDC| 0xFC => {
                opcode_type = OpcodeType::NOP; addr_mode = Rc::new(AddrMode::AbsoluteX); },

            // STP
            0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xB2 | 0xD2 | 0xF2  => {
                opcode_type = OpcodeType::STP; addr_mode = Rc::new(AddrMode::Implied); },

            _ => { opcode_type = OpcodeType::UNK; addr_mode = Rc::new(AddrMode::Implied); }
        };

        let opcode: Opcode = Opcode { opcode_type };
        let addressing: Addressing = Addressing { addr_mode };

        (opcode, addressing)
    }

    fn execute_instruction(&mut self, opcode: Opcode, addressing: Addressing) {
        let addressing_temp = addressing.clone();
        let operand = self.read_operand(addressing);
        let operand_second;
        let mut jmp_flg = false;

        match *addressing_temp.addr_mode {
            AddrMode::Indirect | AddrMode::IndirectX | AddrMode::IndirectY |
            AddrMode::Absolute | AddrMode::AbsoluteX | AddrMode::AbsoluteY => {
                operand_second = self.read_operand(addressing_temp.clone());
            },
            _ => {
                operand_second = None;
            }
        };

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
                        self.cpu_p_reg.set_status_flg(StatusRegister::ZERO_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(StatusRegister::ZERO_FLG);
                    }
                    if (operand_value & T::from(BIN_BIT_7)) != T::from(0) {
                        self.cpu_p_reg.set_status_flg(StatusRegister::NEGATIVE_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(StatusRegister::NEGATIVE_FLG);
                    }
                    if (operand_value & T::from(BIN_BIT_6)) != T::from(0) {
                        self.cpu_p_reg.set_status_flg(StatusRegister::OVERFLOW_FLG);
                    } else {
                        self.cpu_p_reg.cls_status_flg(StatusRegister::OVERFLOW_FLG);
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
                    if self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG) {
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
                    if self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG) {
                        carry = T::from(0x01);
                    }
                    let result: T = a - val - carry;
                    self.set_register(CPUReg::A, result);
                    self.cpu_p_reg.nzv_flg_update(result.try_into().unwrap());
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
                if self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG) {
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
                if self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG) {
                    carry = BIN_BIT_7;
                }
                ret = ret | carry; // bit7 = C Flag Set
                self.cpu_p_reg.nzv_flg_update(ret.try_into().unwrap());
                self.set_register(CPUReg::A, ret.try_into().unwrap());
                println!("ROR");
            }

            // Load/Store Operations
            OpcodeType::LDA => {
                if let Some(value) = operand {
                    let val = value.into();
                    self.set_register(CPUReg::A, val);
                }
                println!("LDA");
            }
            OpcodeType::LDX => {
                if let Some(value) = operand {
                    let val = value.into();
                    self.set_register(CPUReg::X, val);
                }
                println!("LDX");
            }
            OpcodeType::LDY => {
                if let Some(value) = operand {
                    let val = value.into();
                    self.set_register(CPUReg::Y, val);
                }
                println!("LDY");
            }
            OpcodeType::STA => {
                let a: T = self.get_register(CPUReg::A);
                self.write(self.cpu_pc.pc, a);
                println!("STA");
            }
            OpcodeType::STX => {
                let x: T = self.get_register(CPUReg::X);
                self.write(self.cpu_pc.pc, x);
                println!("STX");
            }
            OpcodeType::STY => {
                let y: T = self.get_register(CPUReg::Y);
                self.write(self.cpu_pc.pc, y);
                println!("STY");
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
                println!("PLA");
            }
            OpcodeType::PLP => {
                let value = self.pop_stack();
                self.cpu_p_reg.set_status_flg_all(value.try_into().unwrap());
                println!("PLP");
            }

            // Status Flag Operations / ステータスフラグ関連の命令
            OpcodeType::CLC => {
                self.cpu_p_reg.cls_status_flg(StatusRegister::CARRY_FLG);
                println!("CLC");
            }
            OpcodeType::CLD => {
                self.cpu_p_reg.cls_status_flg(StatusRegister::DECIMAL_MODE_FLG);
                println!("CLD");
            }
            OpcodeType::CLI => {
                self.cpu_p_reg.cls_status_flg(StatusRegister::INTERRUPT_DISABLE_FLG);
                println!("CLI");
            }
            OpcodeType::CLV => {
                self.cpu_p_reg.cls_status_flg(StatusRegister::OVERFLOW_FLG);
                println!("CLV");
            }
            OpcodeType::SEC => {
                self.cpu_p_reg.set_status_flg(StatusRegister::CARRY_FLG);
                println!("SEC");
            }
            OpcodeType::SED => {
                self.cpu_p_reg.set_status_flg(StatusRegister::DECIMAL_MODE_FLG);
                println!("SED");
            }
            OpcodeType::SEI => {
                self.cpu_p_reg.set_status_flg(StatusRegister::INTERRUPT_DISABLE_FLG);
                println!("SEI");
            }

            // Jump and Call Operations
            OpcodeType::JMP => {
                if let Some(value) = operand {
                    let val: u16 = value.into();
                    let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                    let jump_addr = val | (val_second << 8);
                    self.cpu_pc.pc = jump_addr;
                    println!("JMP ${:04X}", jump_addr);
                }
                jmp_flg = true;
            }
            OpcodeType::JSR => {
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let return_addr: T = self.cpu_pc.pc.try_into().unwrap();
                self.push_stack(return_addr);

                if let Some(value) = operand {
                    let val: u16 = value.into();
                    let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                    let jump_addr = val | (val_second << 8);
                    self.cpu_pc.pc = jump_addr;
                    println!("JSR ${:04X}", jump_addr);
                }
                jmp_flg = true;
            }

            // Branch Operations / 分岐命令
            OpcodeType::BCC => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BCC ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BCC Not Jump!");
            }
            OpcodeType::BCS => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::CARRY_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BCS ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BCS Not Jump!");
            }
            OpcodeType::BEQ => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::ZERO_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BEQ ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BEQ Not Jump!");
            }
            OpcodeType::BNE => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::ZERO_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BNE ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BNE Not Jump!");
            }
            OpcodeType::BVC => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::OVERFLOW_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BVC ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BVC Not Jump!");
            }
            OpcodeType::BVS => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::OVERFLOW_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BVS ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BVS Not Jump!");
            }
            OpcodeType::BPL => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::NEGATIVE_FLG);
                if ret != true {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BPL ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BPL Not Jump!");
            }
            OpcodeType::BMI => {
                let ret = self.cpu_p_reg.get_status_flg(StatusRegister::NEGATIVE_FLG);
                if ret != false {
                    if let Some(value) = operand {
                        let val: u16 = value.into();
                        let val_second: u16 = operand_second.unwrap_or(T::from(0x00)).try_into().unwrap();
                        let jump_addr = val | (val_second << 8);
                        self.cpu_pc.pc = jump_addr;
                        println!("BMI ${:04X}", jump_addr);
                    }
                    jmp_flg = true;
                }
                println!("BMI Not Jump!");
            }

            // Intrrupt Operations / 割込み関連
            OpcodeType::RTI => {
                // TODO RTI実装
                println!("RTI");
            }
            OpcodeType::RTS => {
                // TODO RTS実装
                println!("RTS");
            }
            OpcodeType::BRK => {
                // PC++してフラグBとⅠを立てる
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                self.cpu_p_reg.set_status_flg(StatusRegister::BREAK_COMMAND_FLG);
                self.cpu_p_reg.set_status_flg(StatusRegister::INTERRUPT_DISABLE_FLG);
                // スタックにPCとステータスを退避
                self.push_stack(self.cpu_pc.pc.try_into().unwrap());
                self.push_stack(self.cpu_p_reg.get_status_flg_all().try_into().unwrap());
                // TODO IRQのベクタテーブル(0xFFFE,0xFFFE)の2バイトをPCにセット
                // TODO IRQのベクタテーブル(0xFFFE)にPCセット
                // 上の処理は結局、0x8000に飛ぶからもうここで直接PCをいじっとく
                self.cpu_pc.pc = ProgramCounter::ADDR_PRG_ROM;
                println!("BRK");
            }

            // Other
            OpcodeType::STP | _ => {
                // TODO STPと未定義命令をどうするか
                println!("Undefined Instruction!");
            }
        }

        // pc ++
        if jmp_flg != true {
            self.cpu_pc.pc = self.cpu_pc.pc + 1;
        }

    }

    fn push_stack(&mut self, data: T) {
        let sp = self.get_register(CPUReg::SP);
        let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
        self.write(address, data);
        self.set_register(CPUReg::SP, sp - T::from(1u8));
    }

    fn pop_stack(&mut self) -> T {
        let sp = self.get_register(CPUReg::SP);
        self.set_register(CPUReg::SP, sp + T::from(1u8));
        let address: u16 = 0x0100u16.wrapping_add(sp.try_into().unwrap());
        self.read(address)
    }

    fn read_operand(&mut self, addressing: Addressing) -> Option<T>
    {
        match *addressing.addr_mode {
            AddrMode::Accumulator => {
                // アキュムレータモードではオペランドが不要
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                // アキュムレータレジスタの値を返す
                Some(self.get_register(CPUReg::A))
            }
            AddrMode::Immediate => {
                // イミディエイトモードでは次のバイトが即値データ
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                Some(self.read(self.cpu_pc.pc))
            }
            AddrMode::Absolute => {
                // アブソリュートモードでは次の2バイトが絶対アドレス
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                Some(self.read(self.cpu_pc.pc))
            }
            AddrMode::ZeroPage => {
                // ゼロページモードでは次のバイトがゼロページアドレス
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                Some(self.read(self.cpu_pc.pc))
            }
            AddrMode::ZeroPageX => {
                // ゼロページ、Xインデックスモードでは次のバイトがゼロページアドレスとXレジスタの値の和
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                Some(self.read(address.try_into().unwrap()))
            }
            AddrMode::ZeroPageY => {
                // ゼロページ、Yインデックスモードでは次のバイトがゼロページアドレスとYレジスタの値の和
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                Some(self.read(address.try_into().unwrap()))
            }
            AddrMode::AbsoluteX => {
                // アブソリュート、Xインデックスモードでは次の2バイトが絶対アドレスとXレジスタの値の和
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                Some(self.read(address.try_into().unwrap()))
            }
            AddrMode::AbsoluteY => {
                // アブソリュート、Yインデックスモードでは次の2バイトが絶対アドレスとYレジスタの値の和
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let address = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));

                Some(self.read(address.try_into().unwrap()))
            }
            AddrMode::Indirect => {
                // インダイレクトモードでは次の2バイトがジャンプ先の絶対アドレスを格納しているアドレス
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let indirect_address: T = self.read(self.cpu_pc.pc);
                Some(self.read(indirect_address.try_into().unwrap()))
            }
            AddrMode::IndirectX => {
                // インデックスインダイレクト、Xインデックスモードでは次のバイトがアドレスの基準となる値
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::X).try_into().unwrap()));
                let indirect_address: T = self.read(base_address.try_into().unwrap());
                Some(self.read(indirect_address.try_into().unwrap()))
            }
            AddrMode::IndirectY => {
                // インダイレクトインデックス、Yインデックスモードでは次のバイトがアドレスの基準となる値
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let base_address: T = self.read(self.cpu_pc.pc.wrapping_add(self.get_register(CPUReg::Y).try_into().unwrap()));
                let indirect_address: T = self.read(base_address.try_into().unwrap());
                Some(self.read(indirect_address.try_into().unwrap()))
            }
            AddrMode::Relative => {
                // リラティブモードでは次のバイトが相対的なジャンプオフセット
                self.cpu_pc.pc = self.cpu_pc.pc + 1;
                let offset = self.read(self.cpu_pc.pc);
                let target_address: u16 = self.cpu_pc.pc.wrapping_add(offset.try_into().unwrap());
                Some(self.read(target_address.try_into().unwrap()))
            }
            AddrMode::Implied => {
                // インプライドモードではオペランドが存在しない
                None
            }
        }
    }
}

fn main()
{
    let _nes_thread = thread::spawn(|| {
        let mut cpu = RP2A03 {
            cpu_reg: [0u8; 4],
            cpu_p_reg: StatusRegister::new(),
            cpu_pc: ProgramCounter::new(),
            nes_mem: NESMemory::new(),
        };

        // LDA #$c0 TAX INX BRK(a9 c0 aa e8 00)
        cpu.nes_mem.prg_rom.push(0xA9);
        cpu.nes_mem.prg_rom.push(0xC0);
        cpu.nes_mem.prg_rom.push(0xAA);
        cpu.nes_mem.prg_rom.push(0xE8);
        cpu.nes_mem.prg_rom.push(0x00);
        println!("[TEST] : ROM = {:02X?}", cpu.nes_mem.prg_rom);

        cpu.reset();
        println!("[DEBUG] : RP2A03 Inited!");

        loop
        {
            let a: u8 = cpu.get_register(CPUReg::A);
            let x: u8 = cpu.get_register(CPUReg::X);
            let y: u8 = cpu.get_register(CPUReg::Y);
            let sp: u8 = cpu.get_register(CPUReg::SP);
            let p: u8 = cpu.cpu_p_reg.get_status_flg_all();
            let pc: u16 = cpu.cpu_pc.pc;
            println!("[DEBUG] A:0x{:02X},X:0x{:02X},Y:0x{:02X},S:0x{:02X},P:{:08b},PC:0x{:04X}",a,x,y,sp,p,pc);

            println!("[DEBUG] : Fetch!");
            let op_code = cpu.fetch_instruction();
            println!("[DEBUG] : Decode!");
            let (opcode, addressing) = cpu.decode_instruction(op_code);
            println!("[DEBUG] : Execute!");
            cpu.execute_instruction(opcode, addressing);

            // thread::sleep(Duration::from_millis(500)); // 500nsec
            thread::sleep(Duration::from_secs(1));     // 5nsec
        }
    });

    loop {
        // TODO アプリ処理
    }
}


// ====================================== TEST ======================================
#[cfg(test)]
mod cpu_test {
    use super::*;

    // [単体テスト... メモリRead、レジスタW/R、フェッチ、デコード、実行]
    #[test]
    fn cpu_test_func()
    {
        let mut cpu = RP2A03 {
            cpu_reg: [0u8; 4],
            cpu_p_reg: StatusRegister::new(),
            cpu_pc: ProgramCounter::new(),
            nes_mem: NESMemory::new(),
        };

        // [Test Asm] SEC, SED, SEI, CLC, CLD, CLI, CLV
        //      0) 初期状態（bit5と、Vフラグが立っている）:     0110_0000
        //      1) SEC（キャリーフラグをセット）:              0110_0001
        //      1) SED（デシマルモードフラグをセット）:         0110_0011
        //      1) SEI（割り込み無効フラグをセット）:           0110_0111
        //      2) CLC（キャリーフラグをクリア）:              0110_0110
        //      2) CLD（デシマルモードフラグをクリア）:         0110_0100
        //      2) CLI（割り込み無効フラグをクリア）:           0110_0000
        //      2) CLV（オーバーフローフラグをクリア）:         0010_0000
        cpu.cpu_p_reg.set_status_flg(StatusRegister::OVERFLOW_FLG);
        cpu.nes_mem.prg_rom.extend([0x38, 0xF8, 0x78, 0x18, 0xD8, 0x58, 0xB8].iter().cloned());

        // ; [Test Asm]
        // LDA #$0A ; A = 0x0A
        // TAX      ; A = 0, X = 0x0A
        // TXA      ; A = 0x0A, X = 0
        //
        // LDA #$0B ; A = 0x0B
        // TAY      ; A = 0, X:0x0A, Y = 0x0B
        // TYA      ; A = 0x0B, X:0x0A, Y = 0
        cpu.nes_mem.prg_rom.extend([0xA9, 0x0A, 0xAA, 0x8A, 0xA9, 0x0B, 0xA8, 0x98].iter().cloned());


        // ; [Test Asm]
        //          ; A = 0x0B,
        // ORA #$A0 ; A = 0xAB (0xA0 | 0x0B = 0xAB)
        // EOR #$BA ; A = 0x11 (0xAB ^ 0xBA = 0x11)
        // AND #$44 ; A = 0x00 (0x44 & 0x11 = 0x00)
        cpu.nes_mem.prg_rom.extend([0x09, 0xA0, 0x49, 0xBA, 0x29, 0x44].iter().cloned());

        // [Test Asm] JMP $8000
        cpu.nes_mem.prg_rom.extend([0x4C, 0x00, 0x80].iter().cloned());

        // ROM Dump
        println!("[TEST] : ROM = {:02X?}", cpu.nes_mem.prg_rom);
        let len = cpu.nes_mem.prg_rom.len();

        for _ in 1..len
        {
            println!("\n[TEST] : Fetch!");
            let op_code = cpu.fetch_instruction();
            println!("[TEST] : Decode!");
            let (opcode, addressing) = cpu.decode_instruction(op_code);
            println!("[TEST] : Execute!");
            cpu.execute_instruction(opcode, addressing);
            let a: u8 = cpu.get_register(CPUReg::A);
            let x: u8 = cpu.get_register(CPUReg::X);
            let y: u8 = cpu.get_register(CPUReg::Y);
            let sp: u8 = cpu.get_register(CPUReg::SP);
            let p: u8 = cpu.cpu_p_reg.get_status_flg_all();
            let pc: u16 = cpu.cpu_pc.pc;
            println!("[TEST] A:0x{:02X},X:0x{:02X},Y:0x{:02X},S:0x{:02X},P:{:08b},PC:0x{:04X}",a,x,y,sp,p,pc);
        }
        let a: u8 = cpu.get_register(CPUReg::A);
        let x: u8 = cpu.get_register(CPUReg::X);
        let y: u8 = cpu.get_register(CPUReg::Y);
        let sp: u8 = cpu.get_register(CPUReg::SP);
        let p: u8 = cpu.cpu_p_reg.get_status_flg_all();
        assert_eq!(p,0b0010_0000, "[ERR]: Test Fail ... Status Reg, Not Match!");
        assert_eq!(x,0x0A, "[ERR]: Test Fail ... X Reg, Not Match!");
        assert_eq!(y,0x0B, "[ERR]: Test Fail ... Y Reg, Not Match!");
        assert_eq!(a,0x00, "[ERR]: Test Fail ... A Reg, Not Match!");
    }
}
// ==================================================================================