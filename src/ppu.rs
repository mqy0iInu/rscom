use crate::mem::*;
use crate::cpu::*;
use std::pin::Pin;
use once_cell::sync::Lazy;

// ==================================================================================
// [PPU Register]
const PPU_REG_PPUCTRL: u16                       = 0x2000;
const PPU_REG_PPUMASK: u16                       = 0x2001;
const PPU_REG_PPUSTATUS: u16                     = 0x2002;
const PPU_REG_OAMADDR: u16                       = 0x2003;
const PPU_REG_OAMDATA: u16                       = 0x2004;
const PPU_REG_PPUSCROLL: u16                     = 0x2005;
const PPU_REG_PPUADDR: u16                       = 0x2006;
const PPU_REG_PPUDATA: u16                       = 0x2007;
// const PPU_REG_OAMDMA: u16                        = 0x4014;

// [PPUCTRL Bits]
const REG_PPUCTRL_BIT_GENERATE_NMI: u8           = 0b10000000; // Bit7: NMI生成 (0: オフ, 1: オン)
const REG_PPUCTRL_BIT_MASTER_SLAVE_SELECT: u8    = 0b01000000; // Bit6: マスタ/スレーブ選択(1固定)
const REG_PPUCTRL_BIT_SPRITE_SIZE: u8            = 0b00100000; // Bit5: スプライトサイズ
const REG_PPUCTRL_BIT_BACKROUND_PATTERN_ADDR: u8 = 0b00010000; // Bit4: 背景パターンテーブルアドレス
const REG_PPUCTRL_BIT_SPRITE_PATTERN_ADDR: u8    = 0b00001000; // Bit3: スプライトパターンテーブルアドレス
const REG_PPUCTRL_BIT_VRAM_ADD_INCREMENT: u8     = 0b00000100; // Bit2: VRAMアドレスインクリメント (0: 1++, 1: 32++)
const REG_PPUCTRL_BIT_NAMETABLE2: u8             = 0b00000010; // Bit1: 名前テーブル2
const REG_PPUCTRL_BIT_NAMETABLE1: u8             = 0b00000001; // Bit0: 名前テーブル1

// [PPUMASK Bits]
const REG_PPUMASK_BIT_BG_COLOR: u8               = 0b11100000; // Bit7-5: 背景色
const BG_COLOR_RED: u8                           = 0b100;      // 背景色 - 赤
const BG_COLOR_GREEN: u8                         = 0b010;      // 背景色 - 緑
const BG_COLOR_BLUE: u8                          = 0b001;      // 背景色 - 青
const BG_COLOR_BLACK: u8                         = 0b000;      // 背景色 - ブラック
const REG_PPUMASK_BIT_SPRITE_ENABLE: u8          = 0b00010000; // Bit4: スプライト表示 (0: オフ, 1: オン)
const REG_PPUMASK_BIT_BACKGROUND_ENABLE: u8      = 0b00001000; // Bit3: 背景表示 (0: オフ, 1: オン)
const REG_PPUMASK_BIT_SPRITE_LEFT_COLUMN: u8     = 0b00000100; // Bit2: スプライトマスク、画面左8ピクセルを描画しない。(0:描画しない、1:描画)
const REG_PPUMASK_BIT_BACKGROUND_LEFT_COLUMN: u8 = 0b00000010; // Bit1: 背景マスク、画面左8ピクセルを描画しない。(0:描画しない、1:描画)
const REG_PPUMASK_BIT_GRAYSCALE: u8              = 0b00000001; // Bit0: グレースケール (0: カラー, 1: モノクロ)
const GRAYSCALE_COLOR: u8                        = 0;
// const GRAYSCALE_MONOCHRO: u8                     = 1;

// [PPUSTATUS Bits]
const REG_PPUSTATUS_BIT_VBLANK: u8               = 0b10000000; // Bit7: VBLANK状態
const REG_PPUSTATUS_BIT_SPRITE_0_HIT: u8         = 0b01000000; // Bit6: スプライト0ヒット
const REG_PPUSTATUS_BIT_SPRITE_OVERFLOW: u8      = 0b00100000; // Bit5: スプライトオーバーフロー(0:8個以下、1:9個以上)
// const REG_PPUSTATUS_BIT_UNUSED: u8               = 0b00011100; // Bit[4:0]: 未使用

// [OAMADDR/OAMDATA/PPUSCROLL/PPUADDR/PPUDATA/OAMDMA Bits]
// ビット定義なし

// ==================================================================================
// [PPU Memory]
const PPU_OAM_SIZE: usize = 0x0100;
const PPU_OAM_START_ADDR: u16 = 0x0000;
const PPU_OAM_END_ADDR: u16 = 0x00FF;

const PPU_PRAM_SIZE: usize = 0x0020;
const PPU_PRAM_START_ADDR: u16 = 0x3F00;
const PPU_PRAM_END_ADDR: u16 = 0x3F1F;

const VRAM_SIZE: usize = 0x4000;
const VRAM_START_ADDR: u16 = 0x2008;
const VRAM_END_ADDR: u16 = 0x3FFF;
// ==================================================================================
pub const PPU_REG_READ: u8 = 0x00;
pub const PPU_REG_WRITE: u8 = 0x01;

#[derive(Clone)]
pub struct PPU {
    ppuctrl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    oamdata: u8,
    ppuscroll: u8,
    ppuaddr: u8,
    ppudata: u8,
    // oamdma: u8,

    oam: [u8; PPU_OAM_SIZE],
    pram: [u8; PPU_PRAM_SIZE],

    vram: [u8; VRAM_SIZE],
    vram_addr_inc: u8,
    vram_addr_write: u8,
    vram_addr: u16,

    scroll_x: u8,
    scroll_y: u8,
    scroll_write: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            ppuctrl: REG_PPUCTRL_BIT_MASTER_SLAVE_SELECT,
            ppumask: 0,
            ppustatus: 0,
            oamaddr: 0,
            oamdata: 0,
            ppuscroll: 0,
            ppuaddr: 0,
            ppudata: 0,
            // oamdma: 0,

            oam: [0; PPU_OAM_SIZE],
            pram: [0; PPU_PRAM_SIZE],

            vram: [0; VRAM_SIZE],
            vram_addr_inc: 1,
            vram_addr_write: 0,
            vram_addr: 0x2000,

            scroll_x: 0,
            scroll_y: 0,
            scroll_write: 0,
        }
    }

    fn ppu_reg_read(&mut self, address: u16) -> u8 {
        match address {
            PPU_REG_PPUCTRL   => self.ppuctrl,
            PPU_REG_PPUMASK   => self.ppumask,
            PPU_REG_PPUSTATUS => self.ppustatus,
            PPU_REG_OAMADDR   => self.oamaddr,
            PPU_REG_OAMDATA   => self.oamdata,
            PPU_REG_PPUSCROLL => self.ppuscroll,
            PPU_REG_PPUADDR   => self.ppuaddr,
            PPU_REG_PPUDATA   => {
                self.ppudata = self.ppu_mem_read(self.vram_addr);
                self.vram_addr = self.vram_addr.wrapping_add(self.vram_addr_inc as u16);
                self.ppuaddr = (self.vram_addr & 0x00FF) as u8;
                self.ppudata
            },
            // PPU_REG_OAMDMA    => self.oamdma,
            _ => panic!("[PPU Read]: Invalid PPU Register Address: 0x{:04X}", address),
        }
    }

    fn ppu_reg_write(&mut self, address: u16, data: u8) {
        match address {
            PPU_REG_PPUCTRL   => self.ppuctrl = data | REG_PPUCTRL_BIT_MASTER_SLAVE_SELECT,
            PPU_REG_PPUMASK   => self.ppumask = data,
            PPU_REG_PPUSTATUS => self.ppustatus = data,
            PPU_REG_OAMADDR   => self.oamaddr = data,
            PPU_REG_OAMDATA   => {
                self.oamdata = data;
                self.ppu_mem_write(self.oamaddr as u16, self.oamdata);
                self.oamaddr = self.oamaddr.wrapping_add(1);
            },
            PPU_REG_PPUSCROLL => {
                self.ppuscroll = data;
                if self.scroll_write == 0 {
                    self.scroll_x = data;
                    self.scroll_write += 1;
                }else{
                    self.scroll_y = data;
                    self.scroll_write = 0;
                }
            },
            PPU_REG_PPUADDR   => {
                self.ppuaddr = data;
                if self.vram_addr_write == 0 {
                    self.vram_addr = (self.ppuaddr as u16) << 8;
                    self.vram_addr_write += 1;
                }else{
                    self.vram_addr |= self.ppuaddr as u16;
                    self.vram_addr_write = 0;
                }
            },
            PPU_REG_PPUDATA   => {
                self.ppudata = data;
                self.ppu_mem_write(self.vram_addr, self.ppudata);
                self.vram_addr = self.vram_addr.wrapping_add(self.vram_addr_inc as u16);
                self.ppuaddr = (self.vram_addr & 0x00FF) as u8;
            },
            // PPU_REG_OAMDMA    => self.oamdma = data,
            _ => panic!("[PPU Write]: Invalid PPU Register Address: 0x{:04X}", address),
        }
    }

    pub fn ppu_reg_ctrl(&mut self, addr: u16, wr: u8, data: u8) -> u8
    {
        if wr != PPU_REG_WRITE {
            self.ppu_reg_write(addr, data);
            0
        }else{
            self.ppu_reg_read(addr)
        }
    }

    // pub fn ppu_oam_read(&mut self, addr: u8) -> u8
    // {
    //     self.oam[addr as usize]
    // }

    pub fn ppu_oam_write(&mut self, addr: u8, data: u8)
    {
        self.oam[addr as usize] = data;
    }

    fn ppu_mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            // Pattern Table 0 (CHR-ROM)
            0x0000..=0x0FFF => chr_rom_read(addr),
            // Pattern Table 1 (CHR-ROM)
            0x1000..=0x1FFF => chr_rom_read(addr),

            // VRAM
            0x2000..=0x2EFF => self.vram[(addr - 0x2000) as usize],
            // VRAM Mirror
            0x3000..=0x3EFF => self.vram[(addr - 0x3000) as usize],

            // Palette RAM
            0x3F00..=0x3F1F => self.pram[(addr - 0x3F00) as usize],
            // Palette RAM Mirror #1
            0x3F20..=0x3F3F => self.pram[(addr - 0x3F20) as usize],
            // Palette RAM Mirror #2
            0x3F40..=0x3F5F => self.pram[(addr - 0x3F40) as usize],
            // Palette RAM Mirror #3
            0x3F60..=0x3F7F => self.pram[(addr - 0x3F60) as usize],
            // Palette RAM Mirror #4
            0x3F80..=0x3F9F => self.pram[(addr - 0x3F80) as usize],
            // Palette RAM Mirror #5
            0x3FA0..=0x3FBF => self.pram[(addr - 0x3FA0) as usize],
            // Palette RAM Mirror #6
            0x3FC0..=0x3FDF => self.pram[(addr - 0x3FC0) as usize],
            // Palette RAM Mirror #7
            0x3FE0..=0x3FFF => self.pram[(addr - 0x3FE0) as usize],

            // OAM
            0x4000..=0x401F => self.oam[(addr - 0x4000) as usize],
            _ => panic!("Invalid Mem Addr: {:#04X}", addr),
        }
    }

    fn ppu_mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            // VRAM
            0x2000..=0x2EFF => self.vram[(addr - 0x2000) as usize] = data,
            // VRAM Mirror
            0x3000..=0x3EFF => self.vram[(addr - 0x3000) as usize] = data,

            // Palette RAM
            0x3F00..=0x3F1F => self.pram[(addr - 0x3F00) as usize] = data,
            // Palette RAM Mirror #1
            0x3F20..=0x3F3F => self.pram[(addr - 0x3F20) as usize] = data,
            // Palette RAM Mirror #2
            0x3F40..=0x3F5F => self.pram[(addr - 0x3F40) as usize] = data,
            // Palette RAM Mirror #3
            0x3F60..=0x3F7F => self.pram[(addr - 0x3F60) as usize] = data,
            // Palette RAM Mirror #4
            0x3F80..=0x3F9F => self.pram[(addr - 0x3F80) as usize] = data,
            // Palette RAM Mirror #5
            0x3FA0..=0x3FBF => self.pram[(addr - 0x3FA0) as usize] = data,
            // Palette RAM Mirror #6
            0x3FC0..=0x3FDF => self.pram[(addr - 0x3FC0) as usize] = data,
            // Palette RAM Mirror #7
            0x3FE0..=0x3FFF => self.pram[(addr - 0x3FE0) as usize] = data,
            // OAM
            0x4000..=0x401F => self.oam[(addr - 0x4000) as usize] = data,
            _ => panic!("Invalid Mem Addr: {:#04X}", addr),
        }
    }
}


static mut S_PPU: Lazy<Pin<Box<PPU>>> = Lazy::new(|| {
    let ppu = Box::pin(PPU::new());
    ppu
});

fn ppu_vblank_nmi()
{
    cpu_interrupt(InterruptType::NMI);
    print!("[DEBUG]: PPU V-Blank! NMI Generated!");
}

fn ppu_reg_polling()
{
    unsafe {
        // ==========================================================================
        // [PPUCTRL]
        // ==========================================================================
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_VRAM_ADD_INCREMENT) != 0 {
            S_PPU.vram_addr_inc = 32;
        }else{
            S_PPU.vram_addr_inc = 1;
        }

        // ==========================================================================
        // [PPUMASK]
        // ==========================================================================
        let bg_color: u8 = (S_PPU.ppumask & REG_PPUMASK_BIT_BG_COLOR) >> 6;
        match bg_color {
            BG_COLOR_RED => {
                // TODO :背景色　赤
            },
            BG_COLOR_GREEN => {
                // TODO :背景色　緑
            },
            BG_COLOR_BLUE => {
                // TODO :背景色　青
            },
            BG_COLOR_BLACK | _ => {
                // TODO :背景色　黒
            },
        }

        if(S_PPU.ppumask & REG_PPUMASK_BIT_SPRITE_ENABLE) != 0 {
            // TODO :スプライト 表示
        }else{
            // TODO :スプライト 非表示
        }

        if(S_PPU.ppumask & REG_PPUMASK_BIT_BACKGROUND_ENABLE) != 0 {
            // TODO :背景 表示
        }else{
            // TODO :背景 非表示
        }

        if(S_PPU.ppumask & REG_PPUMASK_BIT_SPRITE_LEFT_COLUMN) != 0 {
            // TODO :スプライト画面左8ピクセル 表示
        }else{
            // TODO :スプライト画面左8ピクセル 非表示
        }

        if(S_PPU.ppumask & REG_PPUMASK_BIT_BACKGROUND_LEFT_COLUMN) != 0 {
            // TODO :背景画面左8ピクセル 表示
        }else{
            // TODO :背景画面左8ピクセル 非表示
        }

        if(S_PPU.ppumask & REG_PPUMASK_BIT_GRAYSCALE) != GRAYSCALE_COLOR {
            // TODO :モノクロ表示
        }else{
            // TODO :カラー表示
        }

        // ==========================================================================
        // [PPUSTATUS]
        // ==========================================================================
        // TODO :V-Blank
        if(S_PPU.ppustatus & REG_PPUSTATUS_BIT_VBLANK) != 0 {
            if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_GENERATE_NMI) != 0 {
                ppu_vblank_nmi();
            }
        }

        if(S_PPU.ppustatus & REG_PPUSTATUS_BIT_SPRITE_0_HIT) != 0 {
            // TODO :スプライトヒット
        }

        if(S_PPU.ppustatus & REG_PPUSTATUS_BIT_SPRITE_OVERFLOW) != 0 {
            // TODO :スプライトオーバーフロー 9個以上
        }else{
            // TODO :スプライトオーバーフロー 8個以下
        }
    }
}

pub fn ppu_reset() -> Box<PPU>
{
    unsafe {
        // TODO :PPU Reset
        let ppu_box: Box<PPU> = Box::from_raw(Pin::as_mut(&mut *S_PPU).get_mut());
        ppu_box
    }
}

pub fn ppu_main()
{
    ppu_reg_polling();
}

// ====================================== TEST ======================================
#[cfg(test)]
mod ppu_test {

    #[test]
    fn ppu_test() {
        // TODO : PPU Test
    }
}
// ==================================================================================