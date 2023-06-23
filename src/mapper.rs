use crate::rom::Mirroring;

const BIT_0: u8 = 0x01;
const BIT_1: u8 = 0x02;
const BIT_2: u8 = 0x04;
const BIT_3: u8 = 0x08;
const BIT_4: u8 = 0x10;
const BIT_5: u8 = 0x20;
const BIT_6: u8 = 0x40;
const BIT_7: u8 = 0x80;

pub const MMC_0: u8 = 0;
pub const MMC_1: u8 = 1;
pub const MMC_2: u8 = 2;
pub const MMC_3: u8 = 3;
const MAPPER_1_PRG_ROM_BANK_SIZE: usize = 16 * 1024;
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
const CHR_BANK_17: u8 = 17;
const CHR_BANK_16: u8 = 16;
const CHR_BANK_15: u8 = 15;
const CHR_BANK_14: u8 = 14;
const CHR_BANK_13: u8 = 13;
const CHR_BANK_12: u8 = 12;
const PGR_RAM_BANK_1: u8 = 1;
const PGR_MEM_ROM: u8 = 0;
const PGR_MEM_RAM: u8 = 1;

pub struct Mmc1Reg {
    sp_reg :u8,        // SP（シリアル・パラレル）レジスタ
    shft_reg :u8,      // シフトレジスタ
    shift: u8,
    ctrl_reg_r0 :u8,   // コントロールレジスタ   R0
    ctrl_reg_r1 :u8,   //       〃               R1
    ctrl_reg_r2 :u8,   //       〃               R2
    ctrl_reg_r3 :u8,   //       〃               R3

    // R0 (V-RAMコントロール)
    chr_bank_size :u16, // CHRバンク(4K or 8KB)
    prg_bank_size :u16, // PGRバンクサイズ(16K or 32KB)
    prg_bank_even :u16, // 固定PRGバンク($8000 or $C000)
    display_type: u8,   // 画面(1 or 4画面)
    scroll_mode: Mirroring, // スクロールモード(H or V)

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
            shft_reg :SHFT_REG_INIT_VAL,
            shift: 0,
            sp_reg: 0,
            ctrl_reg_r0: 0,
            ctrl_reg_r1: 0,
            ctrl_reg_r2: 0,
            ctrl_reg_r3: 0,

            chr_bank_size: 0,
            prg_bank_size: 0,
            prg_bank_even: 0,
            display_type: DISPLAY_TYPE_1,
            scroll_mode: Mirroring::VERTICAL,

            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_mem_type: PGR_MEM_ROM,
            prg_bank: 0,
        }
    }

    fn control_reg_write(&mut self, addr: u16, val: u8)
    {
        match addr {
            // コントロールレジスタ0 (V-RAMコントロール)
            0x8000..=0x9FFF => {
                self.ctrl_reg_r0 = val & 0x1F;

                if(val & BIT_4) != 0 {
                    self.chr_bank_size = 4 * 1024;
                }else{
                    self.chr_bank_size = 8 * 1024;
                }

                if(val & BIT_3) != 0 {
                    self.prg_bank_size = 16 * 1024;
                }else{
                    self.prg_bank_size = 32 * 1024;
                }

                if(val & BIT_2) != 0 {
                    self.prg_bank_even = 0x8000;
                }else{
                    self.prg_bank_even = 0xC000;
                }

                if(val & BIT_1) != 0 {
                    self.display_type = DISPLAY_TYPE_4;
                }else{
                    self.display_type = DISPLAY_TYPE_1;
                }

                if(val & BIT_0) != 0 {
                    self.scroll_mode = Mirroring::HORIZONTAL;
                }else{
                    self.scroll_mode = Mirroring::VERTICAL;
                }
            },
            // コントロールレジスタ1 (CHRバンク0)
            0xA000..=0xBFFF => {
                self.ctrl_reg_r1 = val & 0x1F;

                if(val & BIT_4) != 0 {
                    self.chr_bank_0 = CHR_BANK_16;
                }
                if(val & BIT_3) != 0 {
                    self.chr_bank_0 = CHR_BANK_15;
                }
                if(val & BIT_2) != 0 {
                    self.chr_bank_0 = CHR_BANK_14;
                }
                if(val & BIT_1) != 0 {
                    self.chr_bank_0 = CHR_BANK_13;
                }
                if(val & BIT_0) != 0 {
                    if(self.ctrl_reg_r0 & BIT_4) != 0 {
                        self.chr_bank_0 = CHR_BANK_12;
                    }
                }
            },
            // コントロールレジスタ2 (CHRバンク1)
            0xC000..=0xDFFF => {
                self.ctrl_reg_r2 = val & 0x1F;

                if(val & BIT_4) != 0 {
                    self.chr_bank_1 = CHR_BANK_16;
                }

                if(val & BIT_3) != 0 {
                    self.chr_bank_1 = CHR_BANK_15;
                }

                if(val & BIT_2) != 0 {
                    self.chr_bank_1 = CHR_BANK_14;
                }

                if(val & BIT_1) != 0 {
                    self.chr_bank_1 = CHR_BANK_13;
                }

                if(val & BIT_0) != 0 {
                    self.chr_bank_1 = CHR_BANK_13;
                }
            },
            // コントロールレジスタ3 (PRGバンク)
            0xE000..=0xFFFF => {
                self.ctrl_reg_r3 = val & 0x1F;

                if(val & BIT_4) != 0 {
                    self.prg_mem_type = PGR_MEM_RAM;
                }else{
                    self.prg_mem_type = PGR_MEM_ROM;
                }

                if(val & BIT_3) != 0 {
                    if self.prg_mem_type != PGR_MEM_ROM {
                        self.prg_bank = CHR_BANK_17;
                    }else{
                        self.prg_bank = PGR_RAM_BANK_1;
                    }
                }

                if(val & BIT_2) != 0 {
                    self.prg_bank = CHR_BANK_16;
                }

                if(val & BIT_1) != 0 {
                    self.prg_bank = CHR_BANK_15;
                }

                if ((val & BIT_0) != 0) && ((self.ctrl_reg_r0 & BIT_3) != 0) {
                    self.prg_bank = CHR_BANK_14;
                }
            },
            _ => panic!("[ERR] Invalid Addr of MMC1 Ctrl Reg!!!")
        }
    }

    fn shift_reg_proc(&mut self, addr: u16, data :u8){
        self.sp_reg = data & (BIT_7 | BIT_0);

        // bit7のクリアビットが1 = 初期化
        if (self.sp_reg & BIT_7) != 0 {
            self.sp_reg = 0;
            self.shft_reg = SHFT_REG_INIT_VAL;
        } else {
            let val = self.sp_reg & 0x01;
            if self.shift < 5 {
                self.shft_reg = val >> self.shift;
                self.shift += 1;
            // 5右シフトするとき、指定アドレスに値を転送する
            }else {
                self.shft_reg = val.wrapping_shr(1);
                self.control_reg_write(addr, self.shft_reg);
                self.shft_reg = 0;
                self.shift = 0;
            }
        }
    }
}

pub struct MapperMMC {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    bank_select: u8,

    mmc_1_reg: Mmc1Reg,
}

impl MapperMMC {
    pub fn new() -> Self {
        MapperMMC {
            prg_rom: vec![],
            chr_rom: vec![],
            bank_select: 0,
            mapper: 0,

            mmc_1_reg: Mmc1Reg::new(),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if self.bank_select == MMC_1 {
            self.mmc_1_reg.shift_reg_proc(addr, data);
        }else{
            self.bank_select = data;
        }
    }

    fn mmc_1_read(&self, mem: u8, addr: u16) -> u8 {
        let bank_len = MAPPER_1_PRG_ROM_BANK_SIZE;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            // CHR-ROM Bank 0 (SxROM)
            0xA000..=0xBFFF => {
                // bank_select
                let bank = self.bank_select & 0x01;
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            },
            0xC000..=0xFFFF => {
                // 最後のバンク固定
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            },
            _ => panic!("can't be"),
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

    pub fn read_prg_rom(&mut self, mapper: u8, addr: u16) -> u8 {
        match mapper {
            MMC_0 | MMC_2 => self.mmc_2_read(addr),
            MMC_1 => self.mmc_1_read(PRG_ROM, addr),
            MMC_3 => self.mmc_3_prg_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", mapper),
        }
    }

    pub fn read_chr_rom(&mut self, mapper: u8, addr: u16) -> u8 {
        match mapper {
            MMC_1 => self.mmc_1_read(CHR_ROM, addr),
            MMC_3 => self.mmc_3_chr_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", mapper),
        }
    }
}
