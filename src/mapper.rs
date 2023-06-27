use log::{debug, info, trace};
use crate::{common, rom::RomType};
use common::*;
use crate::rom::Mirroring;

pub const MMC_0: u8 = 0;
pub const MMC_1: u8 = 1;
pub const MMC_2: u8 = 2;
pub const MMC_3: u8 = 3;
const MAPPER_2_PRG_ROM_BANK_SIZE: usize = 16 * 1024;
const MAPPER_3_CHR_ROM_BANK_SIZE: usize = 8 * 1024;
const CHR_ROM: u8 = 0;
// const CHR_RAM: u8 = 1;
const PRG_ROM: u8 = 2;
const PRG_RAM_ENABLE: u8 = 0;
const PRG_RAM_DISABLE: u8 = 1;

// [For MMC1]
// 初期値のbit5は5回シフトしてデータを転送する際の検知用（詳細は↓）
// https://www.nesdev.org/wiki/MMC1#SNROM
// http://www43.tok2.com/home/cmpslv/Famic/Fcmp1.htm
const SHFT_REG_INIT_VAL: u8 = 0b0001_0000;
// const DISPLAY_TYPE_1: u8 = 0;
// const DISPLAY_TYPE_4: u8 = 1;
// const PRG_A_17: u8 = 17;
// const PRG_A_16: u8 = 16;
// const PRG_A_15: u8 = 15;
// const PRG_A_14: u8 = 14;
// const CHR_A_16: u8 = 16;
// const CHR_A_15: u8 = 15;
// const CHR_A_14: u8 = 14;
// const CHR_A_13: u8 = 13;
// const CHR_A_12: u8 = 12;
// const PGR_RAM_BANK_1: u8 = 1;
const PGR_MEM_ROM: u8 = 0;
// const PGR_MEM_RAM: u8 = 1;

const IGNORING_LOW_BIT_BANK: u8 = 0;
const FIX_FIRST_BANK: u8 = 1;
const FIX_LAST_BANK: u8 = 2;

pub struct Mmc1Reg {
    pub rom_type: RomType,

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

impl Mmc1Reg {
    pub fn new() -> Self {
        Mmc1Reg {
            rom_type: RomType::SNROM,

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
            0xA000..=0xBFFF => {
                self.ctrl_reg_r1 = val & 0x1F;

                match self.rom_type {
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
                match self.rom_type {
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

pub struct MapperMMC {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    // pub chr_ram: Vec<u8>,
    pub ext_ram: Vec<u8>,
    pub mapper: u8,
    pub is_chr_ram: bool,
    pub is_prg_ram: bool,
    pub rom_type: RomType,
    bank_select: u8,

    pub mmc_1_reg: Mmc1Reg,
}

impl MapperMMC {
    pub fn new() -> Self {
        MapperMMC {
            prg_rom: vec![],
            chr_rom: vec![],
            // chr_ram: vec![0; _MEM_SIZE_8K as usize],
            ext_ram: vec![0; _MEM_SIZE_8K as usize],
            is_chr_ram: false,
            is_prg_ram: false,
            mapper: 0,
            rom_type: RomType::NROM,
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
        let(bank_len, _bank_addr, bank_ops) = self.mmc_1_reg.prg_bank_mode;
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
                // bank_select
                let bank = self.mmc_1_reg.prg_bank;
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