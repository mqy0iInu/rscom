// use crate::mem::*;
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
const REG_PPUCTRL_BIT_NAMETABLE: u8              = 0b00000011; // Bit[1:0]: 名前テーブル0～3

const SPRITE_SIZE_8X8: u8                        = 0;
const SPRITE_SIZE_8X16: u8                       = 1;
const CHR_ROM_BG_PATTERN_TABLE_0: u16            = 0x0000;
const CHR_ROM_BG_PATTERN_TABLE_1: u16            = 0x1000;
const CHR_ROM_SPRITE_PATTERN_TABLE_0: u16        = 0x0000;
const CHR_ROM_SPRITE_PATTERN_TABLE_1: u16        = 0x1000;
const VRAM_INCREMENT_1: u8                       = 1;
const VRAM_INCREMENT_32: u8                      = 32;
const NAME_TABLE_0: u16                          = 0x2000;
const NAME_TABLE_1: u16                          = 0x2400;
const NAME_TABLE_2: u16                          = 0x2800;
const NAME_TABLE_3: u16                          = 0x2C00;

// [PPUMASK Bits]
const REG_PPUMASK_BIT_BG_COLOR: u8               = 0b11100000; // Bit7-5: 背景色
const REG_PPUMASK_BIT_SPRITE_ENABLE: u8          = 0b00010000; // Bit4: スプライト表示 (0: オフ, 1: オン)
const REG_PPUMASK_BIT_BACKGROUND_ENABLE: u8      = 0b00001000; // Bit3: 背景表示 (0: オフ, 1: オン)
const REG_PPUMASK_BIT_SPRITE_LEFT_COLUMN: u8     = 0b00000100; // Bit2: スプライトマスク、画面左8ピクセルを描画しない。(0:描画しない、1:描画)
const REG_PPUMASK_BIT_BACKGROUND_LEFT_COLUMN: u8 = 0b00000010; // Bit1: 背景マスク、画面左8ピクセルを描画しない。(0:描画しない、1:描画)
const REG_PPUMASK_BIT_GRAYSCALE: u8              = 0b00000001; // Bit0: グレースケール (0: カラー, 1: モノクロ)

const BG_COLOR_RED: u8                           = 0b100;      // 背景色 - 赤
const BG_COLOR_GREEN: u8                         = 0b010;      // 背景色 - 緑
const BG_COLOR_BLUE: u8                          = 0b001;      // 背景色 - 青
const BG_COLOR_BLACK: u8                         = 0b000;      // 背景色 - ブラック
const GRAYSCALE_COLOR: u8                        = 0;          // グレースケール: カラー
const GRAYSCALE_MONOCHRO: u8                     = 1;          // グレースケール: 白黒

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

    pub oamdma_run: bool,
    pub oamdma_done: bool,
    pub oam: [u8; PPU_OAM_SIZE],
    pram: [u8; PPU_PRAM_SIZE],

    nmi_gen: bool,
    master_slave: u8,
    cycle: u16,

    vram: [u8; VRAM_SIZE],
    vram_addr_inc: u8,
    vram_addr_write: u8,
    vram_addr: u16,

    scroll_x: u8,
    scroll_y: u8,
    scroll_write: u8,

    sprite_size: u8,
    bg_pattern_tbl: u16,
    sprite_pattern_tbl: u16,
    name_table: u16,

    bg_color: u8,
    sprite_enable: bool,
    bg_enable: bool,
    sprite_left_enable: bool,
    bg_left_enable: bool,
    grayscale: u8,
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

            oamdma_run: false,
            oamdma_done: false,
            oam: [0; PPU_OAM_SIZE],
            pram: [0; PPU_PRAM_SIZE],

            nmi_gen: false,
            master_slave: 0,
            cycle: 0,

            vram: [0; VRAM_SIZE],
            vram_addr_inc: VRAM_INCREMENT_1,
            vram_addr_write: 0,
            vram_addr: 0x2000,

            scroll_x: 0,
            scroll_y: 0,
            scroll_write: 0,

            sprite_size: SPRITE_SIZE_8X8,
            bg_pattern_tbl: CHR_ROM_BG_PATTERN_TABLE_0,
            sprite_pattern_tbl: CHR_ROM_SPRITE_PATTERN_TABLE_0,
            name_table: NAME_TABLE_0,

            bg_color: BG_COLOR_BLACK,
            sprite_enable: false,
            bg_enable: false,
            sprite_left_enable: false,
            bg_left_enable: false,
            grayscale: GRAYSCALE_COLOR,
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

fn reg_polling()
{
    unsafe {
        // ==========================================================================
        // [PPUCTRL]
        // ==========================================================================
        // bit 7
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_GENERATE_NMI) != 0 {
            S_PPU.nmi_gen = true;
        }else{
            S_PPU.nmi_gen = false;
        }

        // bit 6
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_MASTER_SLAVE_SELECT) != 0 {
            S_PPU.master_slave = 1;
        }else{
            S_PPU.master_slave = 0;
        }

        // bit 5
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_SPRITE_SIZE) != 0 {
            S_PPU.sprite_size = SPRITE_SIZE_8X16;   // 8x16
        }else{
            S_PPU.vram_addr_inc = SPRITE_SIZE_8X8;  // 8x8
        }

        // bit 4
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_BACKROUND_PATTERN_ADDR) != 0 {
            S_PPU.bg_pattern_tbl = CHR_ROM_BG_PATTERN_TABLE_1;   // BG Pattern Tbl 1
        }else{
            S_PPU.bg_pattern_tbl = CHR_ROM_BG_PATTERN_TABLE_0;   // BG Pattern Tbl 0
        }

        // bit 3
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_SPRITE_PATTERN_ADDR) != 0 {
            S_PPU.sprite_pattern_tbl = CHR_ROM_SPRITE_PATTERN_TABLE_1;   // BG Pattern Tbl 1
        }else{
            S_PPU.sprite_pattern_tbl = CHR_ROM_SPRITE_PATTERN_TABLE_0;   // BG Pattern Tbl 0
        }

        // bit 2
        if(S_PPU.ppuctrl & REG_PPUCTRL_BIT_VRAM_ADD_INCREMENT) != 0 {
            S_PPU.vram_addr_inc = VRAM_INCREMENT_32; // +=32
        }else{
            S_PPU.vram_addr_inc = VRAM_INCREMENT_1;  // +=1
        }

        // bit[1:0]
        let name_tbl_bit: u8 = S_PPU.ppuctrl & REG_PPUCTRL_BIT_NAMETABLE;
        match name_tbl_bit {
            0x03     => S_PPU.name_table = NAME_TABLE_3,
            0x02     => S_PPU.name_table = NAME_TABLE_2,
            0x01     => S_PPU.name_table = NAME_TABLE_1,
            0x00 | _ => S_PPU.name_table = NAME_TABLE_0,
        }

        // ==========================================================================
        // [PPUMASK]
        // ==========================================================================
        // bit [7:5]
        let bg_color: u8 = (S_PPU.ppumask & REG_PPUMASK_BIT_BG_COLOR) >> 6;
        match bg_color {
            BG_COLOR_RED       => S_PPU.bg_color = BG_COLOR_RED,
            BG_COLOR_GREEN     => S_PPU.bg_color = BG_COLOR_GREEN,
            BG_COLOR_BLUE      => S_PPU.bg_color = BG_COLOR_BLUE,
            BG_COLOR_BLACK | _ => S_PPU.bg_color = BG_COLOR_BLACK,
        }

        // bit 4
        if(S_PPU.ppumask & REG_PPUMASK_BIT_SPRITE_ENABLE) != 0 {
            S_PPU.sprite_enable = true;
        }else{
            S_PPU.sprite_enable = false;
        }

        // bit 3
        if(S_PPU.ppumask & REG_PPUMASK_BIT_BACKGROUND_ENABLE) != 0 {
            S_PPU.bg_enable = true;
        }else{
            S_PPU.bg_enable = false;
        }

        // bit 2
        if(S_PPU.ppumask & REG_PPUMASK_BIT_SPRITE_LEFT_COLUMN) != 0 {
            S_PPU.sprite_left_enable = true;
        }else{
            S_PPU.sprite_left_enable = false;
        }

        // bit 1
        if(S_PPU.ppumask & REG_PPUMASK_BIT_BACKGROUND_LEFT_COLUMN) != 0 {
            S_PPU.bg_left_enable = true;
        }else{
            S_PPU.bg_left_enable = false;
        }

        // bit 0
        if(S_PPU.ppumask & REG_PPUMASK_BIT_GRAYSCALE) != GRAYSCALE_COLOR {
            S_PPU.grayscale = GRAYSCALE_MONOCHRO;
        }else{
            S_PPU.grayscale = GRAYSCALE_COLOR;
        }
    }
}

// [341クロックで1ライン描画＆次のライン準備]
fn data_set(line: u16)
{
    // TODO :1) 最初の256クロックでBGとスプライトの描画
    for line in 0..32 // 33回実施
    {
        // TODO :1-1)ネームテーブルから1バイトフェッチ
        // TODO :1-2)属性テーブルから1バイトフェッチ
        // TODO :1-3)パターンテーブルから2バイトフェッチ
        // TODO :描画データをSDL2に渡して画面描画
    }

    // TODO :2) 次のスキャンラインで描画されるスプライトの探索
    for line in 0..7 // 8回実施
    {
        // TODO :2-1) スプライトパターンから2バイトのフェッチ
    }
}

// [1フレーム(262本のスキャンライン)の描画]
// ※ 1フレーム = 1/60fps(1/59.94) ≒ 16.668 msec
// ※ (262 x 341クロック) = 89,342クロック
// ※ ≒ 16,639,358.727 nsec ≒ 16.639 msec
// ※T ≒ 186.2434098933431 nsec
fn display_render()
{
    // 最初の240本で画面が描画
    for line in 0..239
    {
        data_set(line);
    }

    // [V-Blak開始]
    // 残22本分 -> 垂直回帰時間
    // ※ = (22 x 341クロック) = 7502クロック ≒ 1,397,198.061 nsec ≒ 1.397 msec
    // ※ 垂直回帰時間 / CPUクロック = 2,500.667 CPUサイクル?
    // ※T ≒ 186.2434098933431 nsec
    // ※1 CPU Clock = 558.7302296800292 nsec
    unsafe {
        S_PPU.ppustatus |= REG_PPUSTATUS_BIT_VBLANK;
    }
}

fn nmi_gen()
{
    cpu_interrupt(InterruptType::NMI);
    print!("[DEBUG]: PPU V-Blank! NMI Generated!");
}

pub fn ppu_reset() -> Box<PPU>
{
    unsafe {
        let ppu_box: Box<PPU> = Box::from_raw(Pin::as_mut(&mut *S_PPU).get_mut());
        // TODO :PPU Reset

        // V-Blak開始
        S_PPU.ppustatus |= REG_PPUSTATUS_BIT_VBLANK;
        ppu_box
    }
}

// NTSC 60FPS（59.94fps）のPPUの処理
pub fn ppu_main()
{
    unsafe {
        if (S_PPU.oamdma_run != true) && (S_PPU.oamdma_done != false) {
            // V-Blak終了
            S_PPU.ppustatus &= !REG_PPUSTATUS_BIT_VBLANK;
            if S_PPU.nmi_gen != false {
                nmi_gen(); // NMI
            }
            // [PPUのお仕事]
            reg_polling();    // レジスタのポーリング
            display_render(); // 画面描画(@SDL2)
            S_PPU.oamdma_done = false;
        }
    }
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