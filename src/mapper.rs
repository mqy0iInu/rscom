use log::{info, warn};
// use log::{debug, error, info, log_enabled, trace, warn, Level};
use crate::{common, rom::RomType};
use common::*;
use crate::rom::Mirroring;

const PRG_RAM_ENABLE: u8 = 0;
const PRG_RAM_DISABLE: u8 = 1;

// [For MMC1]
// 初期値のbit5は5回シフトしてデータを転送する際の検知用（詳細は↓）
// https://www.nesdev.org/wiki/MMC1#SNROM
// http://www43.tok2.com/home/cmpslv/Famic/Fcmp1.htm
const SHFT_REG_INIT_VAL: u8 = 0b0001_0000;
// const PGR_RAM_BANK_1: u8 = 1;
const PGR_MEM_ROM: u8 = 0;
// const PGR_MEM_RAM: u8 = 1;
const IGNORING_LOW_BIT_BANK: u8 = 0;
const FIX_FIRST_BANK: u8 = 1;
const FIX_LAST_BANK: u8 = 2;
const BANK_LAST_2_FIXED: u8 = 0;
const BANK_VARIABLE: u8 = 1;
const TWO_2KB_BANK: u8 = 0;
const FOUR_1KB_BANK: u8 = 1;
const CHR_BANK_1KB: u8 = 0;
const CHR_BANK_2KB: u8 = 1;
const PRG_BANK_8KB: u8 = 2;

pub struct Mapper1 {
    // [レジスタ]
    sp_reg :u8,        // SP（シリアル・パラレル）レジスタ
    shift_reg :u8,     // シフトレジスタ
    shift: u8,
    ctrl_reg_r0 :u8,   // コントロールレジスタ   R0
    ctrl_reg_r1 :u8,   //       〃               R1
    ctrl_reg_r2 :u8,   //       〃               R2
    ctrl_reg_r3 :u8,   //       〃               R3

    // R0 (V-RAMコントロール)
    chr_bank_mode: u16,
    prg_bank_mode: (u16, u16, u8),
    mirror: Mirroring,

    // R1 (CHRバンク0)
    // chr_bank_0: u8,
    r1_chr_ram_bank_4k: u8,
    r1_prg_ram_bank_8k: u8,
    r1_prg_rom_bank_256k: u8,

    // R2 (CHRバンク1)
    // chr_bank_1: u8,
    r2_chr_ram_bank_4k: u8,
    r2_prg_ram_bank_8k: u8,
    r2_prg_rom_bank_256k: u8,

    // R3 (CHRバンク1)
    prg_ram_enable: u8,
    prg_mem_type: u8,
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new() -> Self {
        Mapper1 {
            shift_reg :SHFT_REG_INIT_VAL,
            shift: 0,
            sp_reg: 0,
            ctrl_reg_r0: 0,
            ctrl_reg_r1: 0,
            ctrl_reg_r2: 0,
            ctrl_reg_r3: 0,

            chr_bank_mode: _MEM_SIZE_8K,
            prg_bank_mode: (_MEM_SIZE_32K, 0x8000, IGNORING_LOW_BIT_BANK),
            mirror: Mirroring::VERTICAL,

            r1_chr_ram_bank_4k: 0,
            r1_prg_ram_bank_8k: 0,
            r1_prg_rom_bank_256k: 0,
            r2_chr_ram_bank_4k: 0,
            r2_prg_ram_bank_8k: 0,
            r2_prg_rom_bank_256k: 0,
            // chr_bank_0: 0,
            // chr_bank_1: 0,
            prg_mem_type: PGR_MEM_ROM,
            prg_bank: 0,
            prg_ram_enable: PRG_RAM_DISABLE,
        }
    }

    fn shift_reg_proc(&mut self, addr: u16, data :u8, rom_type: RomType){
        self.sp_reg = data & (_BIT_7 | _BIT_0);
        let val: u8 = self.sp_reg & 0x01;

        // bit7のクリアビットが1 = 初期化
        if (self.sp_reg & _BIT_7) != 0 {
            self.sp_reg = 0;
            self.shift_reg = SHFT_REG_INIT_VAL;
        } else {
            if self.shift < 5 {
                self.shift_reg = (val << 5) | (self.shift_reg >> 1);
                self.shift += 1;
            // 5回右シフトするとき、指定アドレスのレジスタに値を転送
            }else {
                self.shift_reg = (val << 5) | (self.shift_reg.wrapping_shl(1));
                self.control_reg_write(addr, self.shift_reg, rom_type);
                self.sp_reg = SHFT_REG_INIT_VAL;
                self.shift_reg = 0;
                self.shift = 0;
            }
        }
    }

    fn control_reg_write(&mut self, addr: u16, val: u8, rom_type: RomType)
    {
        match addr {
            // コントロールレジスタ0 (V-RAMコントロール)
            0x8000..=0x9FFF => {
                self.ctrl_reg_r0 = val & 0x1F;
                let reg_r0 = self.ctrl_reg_r0;

                self.chr_bank_mode = match (reg_r0 & _BIT_4) >> 4 {
                    1 => _MEM_SIZE_4K,
                    0 | _ => _MEM_SIZE_8K,
                };

                self.prg_bank_mode = match (reg_r0 & (_BIT_3 | _BIT_2)) >> 2 {
                    3 => (_MEM_SIZE_16K, 0xC000, FIX_LAST_BANK),
                    2 => (_MEM_SIZE_16K, 0x8000, FIX_FIRST_BANK),
                    0 | 1 | _ => (_MEM_SIZE_32K, 0x8000, IGNORING_LOW_BIT_BANK),
                };

                self.mirror = match (reg_r0 & (_BIT_1 | _BIT_0)) >> 1 {
                    3 => Mirroring::HORIZONTAL,
                    2 => Mirroring::VERTICAL,
                    1 => Mirroring::ONE_SCREEN_UPPER,
                    0 | _ => Mirroring::ONE_SCREEN_LOWER,
                };
            },
            // コントロールレジスタ1 (CHRバンク0)
            0xA000..=0xBFFF => {
                self.ctrl_reg_r1 = val & 0x1F;

                match rom_type {
                    RomType::SUROM => {
                        // 4bit0
                        // -----
                        // PSSxC
                        // ||| |
                        // ||| +- PPU $0000 で 4 KB CHR RAM バンクを選択 (8 KB モードでは無視)
                        // |++--- 8 KB PRG RAM バンクを選択
                        // +----- 256 KB PRG ROM バンクを選択
                        if self.chr_bank_mode != _MEM_SIZE_8K {
                            self.r1_prg_rom_bank_256k = (self.ctrl_reg_r1 & _BIT_4) >> 4;
                        }
                        self.r1_prg_ram_bank_8k = (self.ctrl_reg_r1 & (_BIT_3 | _BIT_2)) >> 2;
                        self.r1_chr_ram_bank_4k = self.ctrl_reg_r1 & _BIT_0;
                    },
                    _ => panic!("Unknown MMC1 Rom Type"),
                }
            },
            // コントロールレジスタ2 (CHRバンク1)
            0xC000..=0xDFFF => {
                self.ctrl_reg_r2 = val & 0x1F;
                match rom_type {
                    RomType::SUROM => {
                        // 4bit0
                        // -----
                        // PSSxC
                        // ||| |
                        // ||| +- PPU $1000 で 4 KB CHR RAM バンクを選択 (8 KB モードでは無視)
                        // |++--- 8 KB PRG RAM バンクを選択 (8 KB モードでは無視)
                        // +----- 256 KB PRG ROM バンクを選択 ( 8 KB モードでは無視されます)

                        if self.chr_bank_mode != _MEM_SIZE_8K {
                            self.r2_prg_rom_bank_256k = (self.ctrl_reg_r1 & _BIT_4) >> 4;
                            self.r2_prg_ram_bank_8k = (self.ctrl_reg_r1 & (_BIT_3 | _BIT_2)) >> 2;
                            self.r2_chr_ram_bank_4k = self.ctrl_reg_r1 & _BIT_0;
                        }
                    },
                        _ => panic!("Unknown MMC1 Rom Type"),
                }
            },
            // コントロールレジスタ3 (PRGバンク)
            0xE000..=0xFFFF => {
                // 4bit0
                // -----
                // RPPPP
                // |||||
                // |++++- 16KBのPRG ROMバンクを選択（32KBモードではロー・ビットは無視される）
                // +----- MMC1B 以降： PRG RAMチップイネーブル(0: 有効、1: 無効。MMC1Aでは無視)
                //        MMC1A：ビット3は、16Kモードの固定バンク ロジックをバイパスする(0：影響を受ける、1：バイパスされる)
                self.ctrl_reg_r3 = val & 0x1F;

                self.prg_ram_enable = (self.ctrl_reg_r3 & _BIT_4) >> 4;
                if self.prg_bank_mode != (_MEM_SIZE_32K, 0x8000, IGNORING_LOW_BIT_BANK) {
                    self.prg_bank = self.ctrl_reg_r3 & 0x0F;
                }
            },
            _ => panic!("[ERR] Invalid Addr of MMC1 Ctrl Reg!!!")
        }
    }
}

pub struct Mapper4 {
    bank_sel_reg: u8,           // バンクセレクトレジスタ($8000～$9FFEの偶数アドレス)
    bank_data_reg: [u8; 8],     // バンクデータレジスタ  ($8000～$9FFEの奇数アドレス)
    bank_reg_sel: u8,
    chr_bank_mode:    (u8, u8), // ($0000-$0FFF, $1000-$1FFF)
    prg_bank_mode:    (u8, u8), // ($8000-$9FFF, $C000-$DFFF)
    chr_a12_inv_mode: (u8, u8), // ($0000-$0FFF, $1000-$1FFF)

    mirroring_reg: u8,          // ミラーリングレジスタ($A000-$BFFEの偶数アドレス)
    prg_ram_protect_reg: u8,    // PRG-RAM保護レジスタ ($A001-$BFFFの奇数アドレス)
    mirror: Mirroring,
    prg_ram_cs: bool,         // PRG-RAM CS(チップセレクト)
    prg_ram_wp: bool,         // PRG-RAM WP(ライトプロテクト)

    irq_latch_reg: u8,          // IRQラッチレジスタ  ($C000-$DFFEの偶数アドレス)
    irq_reload_reg: u8,         // IRQリロードレジスタ($C001-$DFFFの奇数アドレス)
    irq_reload_flg: bool,

    irq_di_reg: u8,             // IRQ無効レジスタ($E000-$FFFEの偶数アドレス)
    irq_ei_reg: u8,             // IRQ有効レジスタ($E001-$FFFFの奇数アドレス)
    irq_flg: bool,

}

impl Mapper4 {
    pub fn new() -> Self {
        Mapper4 {
            bank_sel_reg: 0,           // バンクセレクトレジスタ($8000～$9FFEの偶数アドレス)
            bank_data_reg: [0; 8],          // バンクデータレジスタ  ($8000～$9FFEの奇数アドレス)
            bank_reg_sel: 0,
            chr_bank_mode:    (CHR_BANK_2KB, CHR_BANK_1KB),
            prg_bank_mode:    (BANK_VARIABLE, BANK_LAST_2_FIXED),
            chr_a12_inv_mode: (TWO_2KB_BANK, FOUR_1KB_BANK),

            mirroring_reg: 0,          // ミラーリングレジスタ($A000-$BFFEの偶数アドレス)
            prg_ram_protect_reg: 0,    // PRG-RAM保護レジスタ ($A001-$BFFFの奇数アドレス)
            mirror: Mirroring::VERTICAL,
            prg_ram_cs: false,         // PRG-RAM CS(チップセレクト)
            prg_ram_wp: false,         // PRG-RAM WP(ライトプロテクト)
            irq_latch_reg: 0,          // IRQラッチレジスタ  ($C000-$DFFEの偶数アドレス)
            irq_reload_reg: 0,         // IRQリロードレジスタ($C001-$DFFFの奇数アドレス)
            irq_reload_flg: false,

            irq_di_reg: 0,             // IRQ無効レジスタ($E000-$FFFEの偶数アドレス)
            irq_ei_reg: 0,             // IRQ有効レジスタ($E001-$FFFFの奇数アドレス)
            irq_flg: false,
        }
    }

    // バンクセレクトレジスタ($8000～$9FFEの偶数アドレス)
    fn bank_sel_reg_write(&mut self, val: u8) {
        // 7  bit  0
        // ---- ----
        // CPMx xRRR
        // |||   |||
        // |||   +++- Specify which bank register to update on next write to Bank Data register
        // |||          000: R0: Select 2 KB CHR bank at PPU $0000-$07FF (or $1000-$17FF)
        // |||          001: R1: Select 2 KB CHR bank at PPU $0800-$0FFF (or $1800-$1FFF)
        // |||          010: R2: Select 1 KB CHR bank at PPU $1000-$13FF (or $0000-$03FF)
        // |||          011: R3: Select 1 KB CHR bank at PPU $1400-$17FF (or $0400-$07FF)
        // |||          100: R4: Select 1 KB CHR bank at PPU $1800-$1BFF (or $0800-$0BFF)
        // |||          101: R5: Select 1 KB CHR bank at PPU $1C00-$1FFF (or $0C00-$0FFF)
        // |||          110: R6: Select 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
        // |||          111: R7: Select 8 KB PRG ROM bank at $A000-$BFFF
        // ||+------- N/A (Nothing on the MMC3, see MMC6)
        // |+-------- PRG ROM bank mode (0: $8000-$9FFF 交換可能,
        // |                                $C000-$DFFF 最後から2番目のバンクに固定;
        // |                             1: $C000-$DFFF 交換可能,
        // |                                $8000-$9FFF 最後から2番目のバンクに固定)
        // +--------- CHR A12の反転     (0: two 2 KB banks at $0000-$0FFF,
        //                                  four 1 KB banks at $1000-$1FFF;
        //                               1: two 2 KB banks at $1000-$1FFF,
        //                                  four 1 KB banks at $0000-$0FFF)
        self.bank_sel_reg = val;
        let reg = self.bank_sel_reg;
        self.bank_sel_reg = reg & 0x07;

        self.chr_a12_inv_mode = match (reg & _BIT_7) >> 7 {
            1     => (FOUR_1KB_BANK, TWO_2KB_BANK),
            0 | _ => (TWO_2KB_BANK, FOUR_1KB_BANK),
        };

        self.prg_bank_mode = match (reg & _BIT_6) >> 6 {
            1     => (BANK_VARIABLE, BANK_LAST_2_FIXED),
            0 | _ => (BANK_LAST_2_FIXED, BANK_VARIABLE),
        };

        self.bank_reg_sel = reg & 0x07;
    }

    fn bank_data_reg_write(&mut self, val: u8) {
        // 7  bit  0
        // ---- ----
        // DDDD DDDD
        // |||| ||||
        // ++++-++++- バンクセレクトレジスタに最後に書き込まれた値に基づく新しいバンク値
        self.bank_data_reg[self.bank_reg_sel as usize] = val;
    }

    fn mirroring_reg_write(&mut self, val: u8) {
        // 7  bit  0
        // ---- ----
        // xxxx xxxM
        //         |
        //         +- Nametable mirroring (0: vertical; 1: horizontal)
        self.mirroring_reg = val;
        self.mirror = match self.mirroring_reg & _BIT_0 {
            1 => Mirroring::HORIZONTAL,
            0 | _ => Mirroring::VERTICAL,
        }
    }

    fn prg_ram_protect_reg_write(&mut self, val: u8) {
        // 7  bit  0
        // ---- ----
        // RWXX xxxx
        // ||||
        // ||++------ N/A(Nothing on the MMC3, see MMC6)
        // |+-------- Write protection (0: allow writes; 1: deny writes)
        // +--------- PRG RAM chip enable (0: disable; 1: enable)
        self.prg_ram_protect_reg = val;
        self.prg_ram_cs = (self.mirroring_reg & _BIT_7) != 0;
        self.prg_ram_wp = (self.mirroring_reg & _BIT_6) != 0;
    }

    fn irq_latch_reg_write(&mut self, val: u8) {
        self.irq_latch_reg = val;
        self.prg_ram_protect_reg = 0; // IRQカウンタをクリア
        // TODO :現在のスキャンラインのPPUサイクル260でリロード???
    }

    fn irq_reload_reg_write(&mut self, val: u8) {
        self.irq_reload_reg = val;
        self.irq_reload_flg = true;
    }

    fn irq_di_reg_write(&mut self, val: u8) {
        // 割込み（IRQ）をマスク
        self.irq_di_reg = val;
        self.irq_flg = false;
    }

    fn irq_ei_reg_write(&mut self, val: u8) {
        // 割込み（IRQ）を許可
        self.irq_ei_reg = val;
        self.irq_flg = true;
    }
}

pub struct Mmc3 {
    pub rom_type: RomType,
    mapper_4: Mapper4,
}

impl Mmc3 {
    pub fn new() -> Self {
        Mmc3 {
            rom_type: RomType::TKROM,
            mapper_4: Mapper4::new(),
        }
    }
}

pub struct Mmc1 {
    pub rom_type: RomType,
    mapper_1: Mapper1,
}

impl Mmc1 {
    pub fn new() -> Self {
        Mmc1 {
            rom_type: RomType::SNROM,
            mapper_1: Mapper1::new(),
        }
    }
}

pub struct MapperMMC {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub chr_ram: Vec<u8>,
    pub ext_ram: Vec<u8>,
    pub mapper: u8,
    pub is_chr_ram: bool,
    pub is_prg_ram: bool,
    pub rom_type: RomType,
    bank_select: u8,

    pub mmc_1: Mmc1,
    pub mmc_3: Mmc3,
}

impl MapperMMC {
    pub fn new() -> Self {
        MapperMMC {
            prg_rom: vec![],
            chr_rom: vec![],
            chr_ram: vec![0; _MEM_SIZE_8K as usize],
            ext_ram: vec![0; _MEM_SIZE_8K as usize],
            is_chr_ram: false,
            is_prg_ram: false,
            mapper: 0,
            rom_type: RomType::NROM,
            bank_select: 0,

            mmc_1: Mmc1::new(),
            mmc_3: Mmc3::new(),
        }
    }

    fn mapper_1_write(&mut self, addr: u16, data: u8)
    {
        match addr {
            // 拡張RAM(WRAM)
            0x6000..=0x7FFF => {
                self.ext_ram[(addr - 0x6000) as usize] = data;
            },
            0x8000..=0xFFFF => {
                self.mmc_1.mapper_1.shift_reg_proc(addr, data, self.rom_type.clone());
            },
            _ => panic!("[ERR] MMC1 Write Addr ${:04X} !!!", addr),
        }
    }

    fn mapper_4_write(&mut self, addr: u16, data: u8)
    {
        // warn!("Mapper 4, Write (Addr: ${:04X}, Val: 0x{:02X})", addr, data);

        match addr {
            // [For PPU]
            // CHR-RAM
            0x0000..=0x1FFF => {
                self.chr_ram[addr as usize] = data;
            },

            // [For CPU]
            // 拡張RAM(WRAM)
            0x6000..=0x7FFF => {
                if (self.mmc_3.mapper_4.prg_ram_cs != true) && (self.mmc_3.mapper_4.prg_ram_wp != true) {
                    self.ext_ram[(addr - 0x6000) as usize] = data;
                }else{
                    warn!("Mapper 4, WRAM Write Protect")
                }
            },
            // レジスタ
            0x8000..=0x9FFF => {
                if (addr % 2) == 0 { // 偶数アドレス
                    self.mmc_3.mapper_4.bank_sel_reg_write(data);
                }else{               // 奇数アドレス
                    self.mmc_3.mapper_4.bank_data_reg_write(data);
                }
            },
            0xA000..=0xBFFF => {
                if (addr % 2) == 0 { // 偶数アドレス
                    self.mmc_3.mapper_4.mirroring_reg_write(data);
                }else{               // 奇数アドレス
                    self.mmc_3.mapper_4.prg_ram_protect_reg_write(data);
                }
            },
            0xC000..=0xDFFF => {
                if (addr % 2) == 0 { // 偶数アドレス
                    self.mmc_3.mapper_4.irq_latch_reg_write(data);
                }else{               // 奇数アドレス
                    self.mmc_3.mapper_4.irq_reload_reg_write(data);
                }
            },
            0xE000..=0xFFFF => {
                if (addr % 2) == 0 { // 偶数アドレス
                    self.mmc_3.mapper_4.irq_di_reg_write(data);
                }else{               // 奇数アドレス
                    self.mmc_3.mapper_4.irq_ei_reg_write(data);
                }
            },
            _ => panic!("[Warrning] MMC4 Write Addr ${:04X} !!!", addr),
        }
    }

    pub fn mmc_1_write(&mut self, addr: u16, data: u8) {
        match self.mapper {
            _MAPPER_1 => self.mapper_1_write(addr, data),
            _ => panic!("[ERR] MMC1 Mapper {}, Write Not Supported", self.mapper)
        }
    }

    pub fn mmc_3_write(&mut self, addr: u16, data: u8) {
        match self.mapper {
            _MAPPER_4 => self.mapper_4_write(addr, data),
            _ => panic!("[ERR] MMC1 Mapper {}, Write Not Supported", self.mapper)
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match self.mapper {
            _MAPPER_1 => self.mmc_1_write(addr, data),
            _MAPPER_4 => self.mmc_3_write(addr, data),
            _ => self.bank_select = data,
        }
    }

    fn mapper_1_read(&self, addr: u16) -> u8 {
        // TODO :Mapper 1 Read (このマッパーだけ激難すぎｗ)
        let(bank_len, _bank_addr, bank_ops) = self.mmc_1.mapper_1.prg_bank_mode;
        let mut first_bank_len: u16 = _MEM_SIZE_16K;
        let mut last_bank_len: u16  = _MEM_SIZE_16K;
        match bank_ops {
            FIX_LAST_BANK => { first_bank_len = bank_len },
            FIX_FIRST_BANK => { last_bank_len = bank_len },
            IGNORING_LOW_BIT_BANK | _ => { first_bank_len = bank_len; last_bank_len = bank_len },
        }
        let bank_max = self.prg_rom.len() / (bank_len as usize);

        match addr {
            // 拡張RAM(WRAM)
            0x6000..=0x7FFF => {
                self.ext_ram[(addr - 0x6000) as usize]
            },
            // PRG-ROM Bank
            0x8000..=0xBFFF => {

                let bank = self.mmc_1.mapper_1.prg_bank;
                self.prg_rom[(addr as usize - 0x8000 + (first_bank_len as usize) * bank as usize) as usize]
            },
            // PRG-ROM Bank(最後のバンク固定)
            0xC000..=0xFFFF => {
                self.prg_rom[(addr as usize - 0xC000 + (last_bank_len as usize) * (bank_max - 1)) as usize]
            },
            _ => panic!("[ERR] MMC1 Read Addr ${:04X} !!!", addr),
        }
    }


    fn mmc_2_read(&self, addr: u16) -> u8 {
        let bank_len = _MEM_SIZE_16K as usize;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x6000..=0x7FFF => {
                self.ext_ram[(addr - 0x6000) as usize]
            },
            0x8000..=0xBFFF => {

                let bank = self.bank_select & 0x0F;
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            },
            0xC000..=0xFFFF => {
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            },
            _ => panic!("[ERR] MMC2 Read Addr ${:04X} !!!", addr),
        }
    }

    fn mapper_4_chr_rom_addr(&self, addr: usize) -> usize {
        let d7 = self.bank_select & 0x80;
        let bank_len: usize = 1 * 1024;
        let r0: usize = (self.mmc_3.mapper_4.bank_data_reg[0] & 0xFE) as usize;
        let r1: usize = (self.mmc_3.mapper_4.bank_data_reg[1] & 0xFE) as usize;
        let r2: usize = (self.mmc_3.mapper_4.bank_data_reg[2] & 0xFF) as usize;
        let r3: usize = (self.mmc_3.mapper_4.bank_data_reg[3] & 0xFF) as usize;
        let r4: usize = (self.mmc_3.mapper_4.bank_data_reg[4] & 0xFF) as usize;
        let r5: usize = (self.mmc_3.mapper_4.bank_data_reg[5] & 0xFF) as usize;

        if d7 == 0 {
            match addr {
                // R0 (2KB)
                0x0000..=0x07FF => addr - (bank_len * 0) + r0 * bank_len,
                // R1 (2KB)
                0x0800..=0x0FFF => addr - (bank_len * 2) + r1 * bank_len,
                // R2
                0x1000..=0x13FF => addr - (bank_len * 4) + r2 * bank_len,
                // R3
                0x1400..=0x17FF => addr - (bank_len * 5) + r3 * bank_len,
                // R4
                0x1800..=0x1BFF => addr - (bank_len * 6) + r4 * bank_len,
                // R5
                0x1C00..=0x1FFF => addr - (bank_len * 7) + r5 * bank_len,
                _ => { warn!("[ERR] Mapper 4 PPU Read Addr ${:04X} !!!", addr); 0 },
            }
        } else {
            match addr {
                // R2
                0x0000..=0x03FF => addr - (bank_len * 0) + r2 * bank_len,
                // R3
                0x0400..=0x07FF => addr - (bank_len * 1) + r3 * bank_len,
                // R4
                0x0800..=0x0BFF => addr - (bank_len * 2) + r4 * bank_len,
                // R5
                0x0C00..=0x0FFF => addr - (bank_len * 3) + r5 * bank_len,
                // R0 (2KB)
                0x1000..=0x17FF => addr - (bank_len * 4) + r0 * bank_len,
                // R1 (2KB)
                0x1800..=0x1FFF => addr - (bank_len * 6) + r1 * bank_len,
                _ => { warn!("[ERR] Mapper 4 PPU Read Addr ${:04X} !!!", addr); 0 },
            }
        }
    }

    fn mapper_4_read(&self, addr: u16) -> u8 {
        // TODO :Mapper4 Read (このマッパー結構難いｗ)
        let bank_len: usize = 8 * 1024;
        let bank_max: usize = self.prg_rom.len() / bank_len;
        let ppu_addr: usize = self.mapper_4_chr_rom_addr(addr as usize);

        match addr {
            // [For PPU]
            0x0000..=0x1FFF => {
                self.chr_rom[ppu_addr]
            },

            // [For CPU]
            0x6000..=0x7FFF => {
                let mut val: u8 = 0;
                if self.mmc_3.mapper_4.prg_ram_cs != true {
                    val = self.ext_ram[(addr - 0x6000) as usize];
                }
                val
            },
            0x8000..=0x9FFF => {
                if (self.mmc_3.mapper_4.bank_sel_reg & _BIT_6) == 0 {
                    // バンク切り替え
                    let bank = self.mmc_3.mapper_4.bank_data_reg[6]; // R6からバンクセレクト
                    self.prg_rom[(addr as usize + bank_len * bank as usize) - 0x8000]
                }else{
                    // 最後から2番目のバンクに固定
                    self.prg_rom[(addr as usize + (bank_max - 2) * bank_len) - 0x8000]
                }
            },
            0xA000..=0xBFFF => {
                let bank = self.mmc_3.mapper_4.bank_data_reg[7]; // R7からバンクセレクト
                self.prg_rom[(addr as usize - bank_len + bank_len * bank as usize) - 0x8000]
            },
            0xC000..=0xDFFF => {
                if (self.mmc_3.mapper_4.bank_sel_reg & _BIT_6) == 0 {
                    // 最後から2番目のバンクに固定
                    self.prg_rom[(addr as usize - (bank_len * 2) + (bank_max - 2) * bank_len) - 0x8000]
                }else{
                    // バンク切り替え
                    let bank = self.mmc_3.mapper_4.bank_data_reg[6]; // R6からバンクセレクト
                    self.prg_rom[(addr as usize - (bank_len * 2) + bank_len * bank as usize) - 0x8000]
                }
            },
            0xE000..=0xFFFF => { // 最後のバンクに固定
                self.prg_rom[(addr as usize - (bank_len * 3) + (bank_max - 1) * bank_len) - 0x8000]
            },
            _ => panic!("[ERR] Mapper 4 Read Addr ${:04X} !!!", addr),
        }
    }

    fn mapper_3_read(&self, addr: u16) -> u8 {
        let bank_len = _MEM_SIZE_8K as usize;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            // [For PPU]
            0x0000..=0x1FFF => {
                let bank = self.bank_select & 0x0F;
                self.chr_rom[(addr as usize + bank_len * bank as usize) as usize]
            },
            // [For CPU]
            0x8000..=0xFFFF => {
                self.prg_rom[(addr - 0x8000)as usize]
            },
            _ => panic!("[ERR] Mapper 3 Read Addr ${:04X} !!!", addr),
        }
    }

    fn mmc_1_read(&self, addr: u16) -> u8 {
        match self.mapper {
            _MAPPER_1 => self.mapper_1_read(addr),
            _ => panic!("[ERR] MMC1 Mapper {}, Read Not Supported", self.mapper)
        }
    }

    fn mmc_3_read(&self, addr: u16) -> u8 {
        match self.mapper {
            _MAPPER_4 => self.mapper_4_read(addr),
            _ => panic!("[ERR] MMC3 Mapper {}, Read Not Supported", self.mapper)
        }
    }

    pub fn read_prg_rom(&mut self, addr: u16) -> u8 {
        match self.mapper {
            _MAPPER_0 | _MAPPER_2 => self.mmc_2_read(addr),
            _MAPPER_1 | _MAPPER_105 | _MAPPER_115 => self.mmc_1_read(addr),
            _MAPPER_3 => self.mapper_3_read(addr),
            _MAPPER_4 | _MAPPER_118 | _MAPPER_119 => self.mmc_3_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", self.mapper),
        }
    }

    pub fn read_chr_rom(&mut self, addr: u16) -> u8 {
        match self.mapper {
            _MAPPER_1 | _MAPPER_105 | _MAPPER_115 => self.mmc_1_read(addr),
            _MAPPER_3 => self.mapper_3_read(addr),
            _MAPPER_4 | _MAPPER_118 | _MAPPER_119 => self.mmc_3_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", self.mapper),
        }
    }

    // pub fn mirror_prg_rom_addr(&self, addr: usize) -> usize
    // {
    //     todo!("mirror_prg_rom_addr() func")
    // }

    pub fn mirror_chr_rom_addr(&mut self, addr: usize) -> usize
    {
        match addr {
            // [For PPU]
            0x0000..=0x1FFF => {
                match self.mapper {
                    _MAPPER_4 => self.mapper_4_chr_rom_addr(addr),
                    _ => panic!("[ERR] Not Emu Support MapperMMC {}", self.mapper),
                }
            },
            0x8000..=0xFFFF => {0},
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ptr_from_vec() {
        let my_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        let ptr: *const u8 = my_vec.as_ptr();

        unsafe {
            let value: u8 = *ptr;
            assert_eq!(value, 1);
        }
    }

    #[test]
    fn test_ptr() {
        let my_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        let ptr: *const u8 = my_vec.as_ptr();

        unsafe {
            for i in 0..my_vec.len() {
                let value: u8 = *ptr.offset(i as isize);
                println!("Value at index {}: {}", i, value);
            }
        }
    }
}