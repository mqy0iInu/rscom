use log::{debug, info, trace};
use crate::opcode::{call, CPU_OPS_CODES};
use crate::bus::{Bus, Mem};

const FLAG_CARRY: u8 = 1 << 0;
const FLAG_ZERO: u8 = 1 << 1;
const FLAG_INTERRRUPT: u8 = 1 << 2;
const FLAG_DECIMAL: u8 = 1 << 3;
const FLAG_BREAK: u8 = 1 << 4;
const FLAG_BREAK2: u8 = 1 << 5; // 5 は未使用。
const FLAG_OVERFLOW: u8 = 1 << 6;
const FLAG_NEGATIVE: u8 = 1 << 7;

const SIGN_BIT: u8 = 1 << 7;

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

    fn get_operand_address(&mut self, _mode: &AddressingMode) -> u16 {
        match _mode {
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
                panic!("_mode {:?} is not supported", _mode);
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
            }

            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            let op = self.find_ops(opscode);
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
        status &= !FLAG_BREAK;
        status |= FLAG_BREAK2;
        self._push(status);

        self.status |= FLAG_INTERRRUPT;
        self.bus.tick(2);
        self.program_counter = self.mem_read_u16(0xFFFA);
    }

    fn apu_irq(&mut self) {
        info!("** APU_IRQ **");

        if self.status & FLAG_INTERRRUPT != 0 {
            return;
        }
        info!("  => CALL");

        self._push_u16(self.program_counter);
        self._push(self.status);
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status |= FLAG_BREAK;
        self.bus.tick(2);
    }

    fn find_ops(&mut self, opscode: u8) -> Option<OpCode> {
        for op in CPU_OPS_CODES.iter() {
            if op.code == opscode {
                return Some(op.clone());
            }
        }
        return None;
    }

    pub fn anc(&mut self, _mode: &AddressingMode) {
        // todo!("anc")
    }
    pub fn arr(&mut self, _mode: &AddressingMode) {
        // todo!("arr")
    }
    pub fn asr(&mut self, _mode: &AddressingMode) {
        // todo!("asr")
    }
    pub fn lxa(&mut self, _mode: &AddressingMode) {
        // todo!("lxa")
    }
    pub fn sha(&mut self, _mode: &AddressingMode) {
        // todo!("sha")
    }
    pub fn sbx(&mut self, _mode: &AddressingMode) {
        //  A&X minus #{imm} into X
        // AND X register with accumulator and store result in X regis-ter, then
        // subtract byte from X register (without borrow).
        // Status flags: N,Z,C

        // AND X をアキュムレータに登録し、結果を X レジスタに格納します。 X レジスタからバイトを減算します (ボローなし)。 ステータスフラグ：N、Z、C
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        let (v, overflow) = (self.register_a & self.register_x).overflowing_sub(value);
        self.register_x = v;
        self.update_zero_and_negative_flags(self.register_x);
        self.status = if overflow {
            self.status & FLAG_OVERFLOW
        } else {
            self.status | FLAG_OVERFLOW
        };
        // todo!("sbx")
    }

    pub fn jam(&mut self, _mode: &AddressingMode) {
        // Stop program counter (processor lock up).
        self.program_counter -= 1;
        // panic!("CALL JAM operation.");
    }

    pub fn lae(&mut self, _mode: &AddressingMode) {
        // stores {adr}&S into A, X and S

        // AND memory with stack pointer, transfer result to accu-mulator, X
        // register and stack pointer.
        // Status flags: N,Z
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        let s = self._pop();
        self.register_a = value & s;
        self.register_x = self.register_a;
        self._push(self.register_a);
        self.update_zero_and_negative_flags(self.register_a);
        // todo!("lae")
    }

    pub fn shx(&mut self, _mode: &AddressingMode) {
        // M =3D X AND HIGH(arg) + 1
        let addr = self.get_operand_address(_mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_x & h).wrapping_add(1));
        // todo!("shx")
    }

    pub fn shy(&mut self, _mode: &AddressingMode) {
        // Y&H into {adr}
        // AND Y register with the high byte of the target address of the argument
        // + 1. Store the result in memory.
        let addr = self.get_operand_address(_mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, (self.register_y & h).wrapping_add(1));
        // todo!("shy")
    }

    pub fn ane(&mut self, _mode: &AddressingMode) {
        // TXA + AND #{imm}
        self.txa(_mode);
        self.and(_mode);
        // todo!("ane")
    }

    pub fn shs(&mut self, _mode: &AddressingMode) {
        // stores A&X into S and A&X&H into {adr}
        // アキュムレータと X レジスタを AND 演算し、結果をスタック ポインタに格納します。次に、スタック ポインタと引数 1 のターゲット アドレスの上位バイトを AND 演算します。結果をメモリに格納します。
        self._push(self.register_a & self.register_x);
        let addr = self.get_operand_address(_mode);
        let h = ((addr & 0xFF00) >> 8) as u8;
        self.mem_write(addr, self.register_a & self.register_x & h);
        // todo!("shs")
    }

    pub fn rra(&mut self, _mode: &AddressingMode) {
        self.ror(_mode);
        self.adc(_mode);
    }

    pub fn sre(&mut self, _mode: &AddressingMode) {
        self.lsr(_mode);
        self.eor(_mode);
    }

    pub fn rla(&mut self, _mode: &AddressingMode) {
        self.rol(_mode);
        self.and(_mode);
    }

    pub fn slo(&mut self, _mode: &AddressingMode) {
        self.asl(_mode);
        self.ora(_mode);
    }

    pub fn isb(&mut self, _mode: &AddressingMode) {
        // = ISC
        self.inc(_mode);
        self.sbc(_mode);
    }

    pub fn dcp(&mut self, _mode: &AddressingMode) {
        self.dec(_mode);
        self.cmp(_mode);
    }

    pub fn sax(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self.mem_write(addr, self.register_a & self.register_x);
    }

    pub fn lax(&mut self, _mode: &AddressingMode) {
        self.lda(_mode);
        self.tax(_mode);
    }

    pub fn txs(&mut self, _mode: &AddressingMode) {
        self.stack_pointer = self.register_x;
    }

    pub fn tsx(&mut self, _mode: &AddressingMode) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn tya(&mut self, _mode: &AddressingMode) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tay(&mut self, _mode: &AddressingMode) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn txa(&mut self, _mode: &AddressingMode) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn tax(&mut self, _mode: &AddressingMode) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn sty(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self.mem_write(addr, self.register_y);
    }

    pub fn stx(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self.mem_write(addr, self.register_x);
    }

    pub fn sta(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self.mem_write(addr, self.register_a);
    }

    pub fn rti(&mut self, _mode: &AddressingMode) {
        // スタックからプロセッサ フラグをプルし、続いてプログラム カウンタをプルします。
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
        self.program_counter = self._pop_u16();
    }

    pub fn plp(&mut self, _mode: &AddressingMode) {
        self.status = self._pop() & !FLAG_BREAK | FLAG_BREAK2;
    }

    pub fn php(&mut self, _mode: &AddressingMode) {
        self._push(self.status | FLAG_BREAK | FLAG_BREAK2);
    }

    pub fn pla(&mut self, _mode: &AddressingMode) {
        self.register_a = self._pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn pha(&mut self, _mode: &AddressingMode) {
        self._push(self.register_a);
    }

    pub fn nop(&mut self, _mode: &AddressingMode) {
        // なにもしない
    }

    pub fn ldy(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn ldx(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn lda(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn rts(&mut self, _mode: &AddressingMode) {
        let value = self._pop_u16() + 1;
        self.program_counter = value;
    }

    pub fn jsr(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self._push_u16(self.program_counter + 2 - 1);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter = self.program_counter.wrapping_sub(2);
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

    pub fn jmp(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        self.program_counter = addr;
        // 後で+2するので整合性のため-2しておく
        self.program_counter -= 2;
        // TODO
        // オリジナルの 6502 は、間接ベクトルがページ境界にある場合、
        // ターゲット アドレスを正しくフェッチしません (たとえば、$xxFF で、xx は $00 から $FF までの任意の値です)。
        // この場合、予想どおり $xxFF から LSB を取得しますが、$xx00 から MSB を取得します。
        // これは、65SC02 などの最近のチップで修正されているため、互換性のために、間接ベクトルがページの最後にないことを常に確認してください。
    }

    pub fn iny(&mut self, _mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn inx(&mut self, _mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn inc(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr).wrapping_add(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn dey(&mut self, _mode: &AddressingMode) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    pub fn dex(&mut self, _mode: &AddressingMode) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    pub fn dec(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr).wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_zero_and_negative_flags(value);
    }

    fn _cmp(&mut self, target: u8, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        if target >= value {
            self.sec(&AddressingMode::Implied);
        } else {
            self.clc(&AddressingMode::Implied);
        }
        let value = target.wrapping_sub(value);
        self.update_zero_and_negative_flags(value);
    }

    pub fn cpy(&mut self, _mode: &AddressingMode) {
        self._cmp(self.register_y, _mode);
    }

    pub fn cpx(&mut self, _mode: &AddressingMode) {
        self._cmp(self.register_x, _mode);
    }

    pub fn cmp(&mut self, _mode: &AddressingMode) {
        self._cmp(self.register_a, _mode);
    }

    pub fn clv(&mut self, _mode: &AddressingMode) {
        self.status = self.status & !FLAG_OVERFLOW;
    }

    pub fn sei(&mut self, _mode: &AddressingMode) {
        self.status = self.status | FLAG_INTERRRUPT;
    }

    pub fn cli(&mut self, _mode: &AddressingMode) {
        self.status = self.status & !FLAG_INTERRRUPT;
    }

    pub fn sed(&mut self, _mode: &AddressingMode) {
        self.status = self.status | FLAG_DECIMAL;
    }

    pub fn cld(&mut self, _mode: &AddressingMode) {
        self.status = self.status & !FLAG_DECIMAL;
    }

    pub fn sec(&mut self, _mode: &AddressingMode) {
        self.status = self.status | FLAG_CARRY;
    }

    pub fn clc(&mut self, _mode: &AddressingMode) {
        self.status = self.status & !FLAG_CARRY;
    }

    pub fn bvs(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_OVERFLOW, true);
    }

    pub fn bvc(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_OVERFLOW, false);
    }

    fn _branch(&mut self, _mode: &AddressingMode, flag: u8, nonzero: bool) {
        let addr = self.get_operand_address(_mode);
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

    pub fn brk(&mut self, _mode: &AddressingMode) {
        if self.status & FLAG_BREAK != 0 {
            return;
        }

        // プログラム カウンターとプロセッサ ステータスがスタックにプッシュされ、
        self._push_u16(self.program_counter);
        self._push(self.status);

        // $FFFE/F の IRQ 割り込みベクトルが PC にロードされ、ステータスのブレーク フラグが 1 に設定されます。
        self.program_counter = self.mem_read_u16(0xFFFE);
        self.status = self.status | FLAG_BREAK;
    }

    pub fn bpl(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_NEGATIVE, false);
    }

    pub fn bmi(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_NEGATIVE, true);
    }

    pub fn bit(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
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

    pub fn bne(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_ZERO, false);
    }

    pub fn beq(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_ZERO, true);
    }

    pub fn bcc(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_CARRY, false);
    }

    pub fn bcs(&mut self, _mode: &AddressingMode) {
        self._branch(_mode, FLAG_CARRY, true);
    }

    pub fn ror(&mut self, _mode: &AddressingMode) {
        let (value, carry) = if _mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            self.register_a = self.register_a | ((self.status & FLAG_CARRY) << 7);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(_mode);
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

    pub fn rol(&mut self, _mode: &AddressingMode) {
        let (value, carry) = if _mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value | (self.status & FLAG_CARRY);
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(_mode);
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

    pub fn lsr(&mut self, _mode: &AddressingMode) {
        let (value, carry) = if _mode == &AddressingMode::Accumulator {
            let carry = self.register_a & 0x01;
            self.register_a = self.register_a / 2;
            (self.register_a, carry)
        } else {
            let addr = self.get_operand_address(_mode);
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

    pub fn asl(&mut self, _mode: &AddressingMode) {
        let (value, carry) = if _mode == &AddressingMode::Accumulator {
            let (value, carry) = self.register_a.overflowing_mul(2);
            self.register_a = value;
            (value, carry)
        } else {
            let addr = self.get_operand_address(_mode);
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

    pub fn ora(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a | value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn eor(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a ^ value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn and(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
        let value = self.mem_read(addr);
        self.register_a = self.register_a & value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    pub fn sbc(&mut self, _mode: &AddressingMode) {
        // A-M-(1-C)
        // キャリーかどうかの判定が逆
        // キャリーの引き算(1-C)
        // overflowの判定が逆 = m,p, p,m
        let addr = self.get_operand_address(_mode);
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

    pub fn adc(&mut self, _mode: &AddressingMode) {
        let addr = self.get_operand_address(_mode);
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
    let ops = cpu.find_ops(op).unwrap();
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
            panic!("_mode {:?} is not supported", ops.addressing_mode);
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