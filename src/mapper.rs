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
const  MMC_1_CLEAR_BIT: u8       = 0x80;
const  MMC_1_SERIAL_DATA_BIT: u8 = 0x00;
// 初期値のbit5は5回シフトしてデータを転送する際の検知用（詳細は↓）
// https://www.nesdev.org/wiki/MMC1#SNROM
// http://www43.tok2.com/home/cmpslv/Famic/Fcmp1.htm
const SHFT_REG_INIT_VAL: u8 = 0b0001_0000;

pub struct Mmc1Reg {
    sp_reg :u8,        // SP（シリアル・パラレル）レジスタ
    shft_reg :u8,      // シフトレジスタ
    shift: u8,
    ctrl_reg_r0 :u8,   // コントロールレジスタ   R0
    ctrl_reg_r1 :u8,   //       〃               R1
    ctrl_reg_r2 :u8,   //       〃               R2
    ctrl_reg_r3 :u8,   //       〃               R3

    // R0 (V-RAMコントロール)
    chr_bank_size :u8,  // CHRバンク(4K or 8KB)
    prg_bank_size :u16, // PGRバンクサイズ(16K or 32KB)
    prg_bank_even :u16, // 固定PRGバンク($8000 or $C000)
    display_type: u8,   // 画面(1 or 4画面)
    scroll_mode: u8,    // スクロールモード(H or V)

    // R1 (CHRバンク0)
    chr_bank_0: u8,

    // R2 (CHRバンク1)
    chr_bank_1: u8,

    // R3 (CHRバンク1)
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
            display_type: 0,
            scroll_mode: 0,

            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    fn control_reg_write(&mut self, addr: u16, val: u8)
    {
        match addr {
            // コントロールレジスタ0 (V-RAMコントロール)
            0x8000..=0x9FFF => {
                self.ctrl_reg_r0 = val & 0x1F;
                // TODO :V-RAMコントロール
            },
            // コントロールレジスタ1 (CHRバンク0)
            0xA000..=0xBFFF => {
                self.ctrl_reg_r1 = val & 0x1F;
                // TODO :CHRバンク0
            },
            // コントロールレジスタ2 (CHRバンク1)
            0xC000..=0xDFFF => {
                self.ctrl_reg_r2 = val & 0x1F;
                // TODO :CHRバンク1
            },
            // コントロールレジスタ3 (PRGバンク)
            0xE000..=0xFFFF => {
                self.ctrl_reg_r3 = val & 0x1F;
                // TODO PRGバンク
            },
            _ => panic!("[ERR] Invalid Addr of MMC1 Ctrl Reg!!!")
        }
    }

    fn shift_reg_proc(&mut self, addr: u16, data :u8){
        self.sp_reg = data & (MMC_1_CLEAR_BIT | MMC_1_SERIAL_DATA_BIT);

        // bit7のクリアビットが1 = 初期化
        if (self.sp_reg & MMC_1_CLEAR_BIT) != 0 {
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
