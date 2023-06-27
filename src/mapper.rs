use log::{debug, info, trace};
use crate::common;
use common::*;
use crate::rom::Mirroring;

pub const MMC_0: u8 = 0;
pub const MMC_1: u8 = 1;
pub const MMC_2: u8 = 2;
pub const MMC_3: u8 = 3;
const MAPPER_2_PRG_ROM_BANK_SIZE: usize = 16 * 1024;
const MAPPER_3_CHR_ROM_BANK_SIZE: usize = 8 * 1024;
const CHR_ROM: u8 = 0;
const CHR_RAM: u8 = 1;
const PRG_ROM: u8 = 2;

// [For MMC1]
// 初期値のbit5は5回シフトしてデータを転送する際の検知用（詳細は↓）
// https://www.nesdev.org/wiki/MMC1#SNROM
// http://www43.tok2.com/home/cmpslv/Famic/Fcmp1.htm
const SHFT_REG_INIT_VAL: u8 = 0b0001_0000;
const DISPLAY_TYPE_1: u8 = 0;
const DISPLAY_TYPE_4: u8 = 1;
const PRG_A_17: u8 = 17;
const PRG_A_16: u8 = 16;
const PRG_A_15: u8 = 15;
const PRG_A_14: u8 = 14;
const CHR_A_16: u8 = 16;
const CHR_A_15: u8 = 15;
const CHR_A_14: u8 = 14;
const CHR_A_13: u8 = 13;
const CHR_A_12: u8 = 12;
const PGR_RAM_BANK_1: u8 = 1;
const PGR_MEM_ROM: u8 = 0;
const PGR_MEM_RAM: u8 = 1;

const IGNORING_LOW_BIT_BANK: u8 = 0;
const FIX_FIRST_BANK: u8 = 1;
const FIX_LAST_BANK: u8 = 2;

pub struct Mmc1Reg {
    sp_reg :u8,        // SP（シリアル・パラレル）レジスタ
    shift_reg :u8,      // シフトレジスタ
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
    chr_bank_0: u8,

    // R2 (CHRバンク1)
    chr_bank_1: u8,

    // R3 (CHRバンク1)
    prg_mem_type: u8,
    prg_bank: u8,
}

impl Mmc1Reg {
    pub fn new() -> Self {
        Mmc1Reg {
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

            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_mem_type: PGR_MEM_ROM,
            prg_bank: 0,
        }
    }

    fn shift_reg_proc(&mut self, addr: u16, data :u8){
        self.sp_reg = data & (_BIT_7 | _BIT_0);
        let val: u8 = self.sp_reg & 0x01;

        // bit7のクリアビットが1 = 初期化
        if (self.sp_reg & _BIT_7) != 0 {
            self.sp_reg = 0;
            self.shift_reg = SHFT_REG_INIT_VAL;
        } else {
            if self.shift < 5 {
                self.shift_reg = (val << _BIT_5) | (self.shift_reg >> 1);
                self.shift += 1;
            // 5回右シフトするとき、指定アドレスのレジスタに値を転送
            }else {
                self.shift_reg = (val << _BIT_5) | (self.shift_reg.wrapping_shl(1));
                self.control_reg_write(addr, self.shift_reg);
                self.sp_reg = SHFT_REG_INIT_VAL;
                self.shift_reg = 0;
                self.shift = 0;
            }
        }
    }

    fn control_reg_write(&mut self, addr: u16, val: u8)
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
            // 4bit0
            // -----
            // PSSxC
            // ||| |
            // ||| +- PPU $0000 で 4 KB CHR RAM バンクを選択 (8 KB モードでは無視)
            // |++--- 8 KB PRG RAM バンクを選択
            // +----- 256 KB PRG ROM バンクを選択
            0xA000..=0xBFFF => {
                self.ctrl_reg_r1 = val & 0x1F;

                if(self.ctrl_reg_r1 & _BIT_4) != 0 {
                    self.chr_bank_0 = CHR_A_16;
                }
            },
            // コントロールレジスタ2 (CHRバンク1)
            // 4bit0
            // -----
            // PSSxC
            // ||| |
            // ||| +- PPU $1000 で 4 KB CHR RAM バンクを選択 (8 KB モードでは無視)
            // |++--- 8 KB PRG RAM バンクを選択 (8 KB モードでは無視)
            // +----- 256 KB PRG ROM バンクを選択 ( 8 KB モードでは無視されます)
            0xC000..=0xDFFF => {
                self.ctrl_reg_r2 = val & 0x1F;

                if(self.ctrl_reg_r2 & _BIT_4) != 0 {
                    self.chr_bank_1 = CHR_A_16;
                }
            },
            // コントロールレジスタ3 (PRGバンク)
            0xE000..=0xFFFF => {
                self.ctrl_reg_r3 = val & 0x1F;

                if(self.ctrl_reg_r3 & _BIT_4) != 0 {
                    self.prg_mem_type = PGR_MEM_RAM;
                }else{
                    self.prg_mem_type = PGR_MEM_ROM;
                }
            },
            _ => panic!("[ERR] Invalid Addr of MMC1 Ctrl Reg!!!")
        }
    }
}

pub struct MapperMMC {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    // pub chr_ram: Vec<u8>,
    pub ext_ram: Vec<u8>,
    pub mapper: u8,
    pub is_ext_ram: bool,
    bank_select: u8,

    mmc_1_reg: Mmc1Reg,
}

impl MapperMMC {
    pub fn new() -> Self {
        MapperMMC {
            prg_rom: vec![],
            chr_rom: vec![],
            // chr_ram: vec![0; _MEM_SIZE_8K as usize],
            ext_ram: vec![0; _MEM_SIZE_8K as usize],
            is_ext_ram: false,
            mapper: 0,
            bank_select: 0,

            mmc_1_reg: Mmc1Reg::new(),
        }
    }

    pub fn mmc_1_write(&mut self, addr: u16, data: u8) {
        trace!("[Trace] MMC1 Write Addr ${:04X}", addr);
        match addr {
            // 拡張RAM(WRAM)
            0x6000..=0x7FFF => {
                self.ext_ram[(addr - 0x6000) as usize];
            },
            0x8000..=0xFFFF => {
                self.mmc_1_reg.shift_reg_proc(addr, data);
                panic!("MMC1 Conf Ref Write");
            },
            _ => panic!("[ERR] MMC1 Write Addr ${:04X} !!!", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if self.mapper == MMC_1 {

        }else{
            self.bank_select = data;
        }
    }

    fn mmc_1_read(&self, mem: u8, addr: u16) -> u8 {
        // TODO :MMC1 (このマッパーだけ激難すぎｗ)
        let mut bank_len = _MEM_SIZE_16K as usize;
        if self.mmc_1_reg.prg_bank != _MEM_SIZE_16K as u8
        {
            bank_len = _MEM_SIZE_32K as usize;
        }
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            // 拡張RAM(WRAM)
            0x6000..=0x7FFF => {
                self.ext_ram[(addr - 0x6000) as usize]
            },
            // PRG-ROM Bank
            0x8000..=0xBFFF => {
                // bank_select
                let bank = self.mmc_1_reg.ctrl_reg_r3 & 0x03;
                trace!("[Trace] MMC1 Read Addr ${:04X}", addr);
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            },
            // PRG-ROM Bank(最後のバンク固定)
            0xC000..=0xFFFF => {
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            },
            _ => panic!("[ERR] MMC1 Read Addr ${:04X} !!!", addr),
        }
    }

    fn mmc_2_read(&self, addr: u16) -> u8 {
        let bank_len = MAPPER_2_PRG_ROM_BANK_SIZE;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x8000..=0xBFFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            },
            0xC000..=0xFFFF => {
                // 最後のバンク固定
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            },
            _ => panic!("can't be"),
        }
    }

    fn mmc_3_prg_rom_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                self.prg_rom[(addr - 0x8000)as usize]
            },
            _ => panic!("can't be"),
        }
    }

    fn mmc_3_chr_rom_read(&self, addr: u16) -> u8 {
        let bank_len = MAPPER_3_CHR_ROM_BANK_SIZE;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x0000..=0x1FFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.chr_rom[(addr as usize + bank_len * bank as usize) as usize]
            },
            _ => panic!("can't be"),
        }
    }

    pub fn read_prg_rom(&mut self, addr: u16) -> u8 {
        match self.mapper {
            MMC_0 | MMC_2 => self.mmc_2_read(addr),
            MMC_1 => self.mmc_1_read(PRG_ROM, addr),
            MMC_3 => self.mmc_3_prg_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", self.mapper),
        }
    }

    pub fn read_chr_rom(&mut self, addr: u16) -> u8 {
        match self.mapper {
            MMC_1 => self.mmc_1_read(CHR_ROM, addr),
            MMC_3 => self.mmc_3_chr_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", self.mapper),
        }
    }
}
