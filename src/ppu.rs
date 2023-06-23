use std::pin::Pin;
use std::boxed::Box;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use sdl2::Sdl;
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowContext};
use sdl2::render::{Texture, TextureCreator};
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use std::io::Write;
use std::sync::Mutex;

// use crate::mem::*;
use crate::cpu::*;
use crate::cassette::*;

const SCREEN_WIDTH : usize = 256;
const SCREEN_HEIGHT: usize = 240;

#[rustfmt::skip]
const COLOR_PALLETE: [(u8,u8,u8); 64] = [
    (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E), (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00), (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05), (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA), (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00), (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF), (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12), (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF), (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF), (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0), (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

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

const VRAM_SIZE: usize = 2048;
const VRAM_START_ADDR: u16 = 0x2000;
const VRAM_END_ADDR: u16 = 0x3FFF;
// ==================================================================================
pub const PPU_REG_READ: u8 = 0x00;
pub const PPU_REG_WRITE: u8 = 0x01;

#[derive(Clone)]
pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HEIGHT) * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

#[derive(Clone, Copy)]
pub struct RenderData {
    pub name_val :u8,
    pub attribute_val :u8,
    pub bg_pattern: [u8; 2],
    pub bg_parette_color: u8,

    pub sprite_y: u8,
    pub sprite_pattern_index :u8,
    pub sprite_attribute: u8,
    pub sprite_x: u8,
    pub sprite_pattern:[u8; 2],
    pub sprite_parette_color: u8,
}

impl RenderData {
    pub fn new() -> Self {
        RenderData {
            name_val :0,
            attribute_val :0,
            bg_pattern :[0; 2],
            bg_parette_color: 0,

            sprite_y: 0,
            sprite_pattern_index :0,
            sprite_attribute: 0,
            sprite_x: 0,
            sprite_pattern :[0; 2],
            sprite_parette_color: 0,
        }
    }
}

struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rect {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Rect {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        }
    }
}

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

    pub render_data: RenderData,
    pub sprite_prefetch_buf: [u8; 16], // 2Byte x 8
    nmi_gen: bool,
    master_slave: u8,
    cycle: u16,

    pub vram: [u8; VRAM_SIZE],
    vram_addr_inc: u8,
    vram_addr_write: u8,
    pub vram_addr: u16,

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

    scanline: u16,
    scanline_palette_indexes: Vec<usize>,
    scanline_palette_tables: Vec<[u8; 32]>,

    freme: Frame,
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

            render_data: RenderData::new(),
            sprite_prefetch_buf: [0; 16],

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

            scanline: 0,
            scanline_palette_indexes: vec![],
            scanline_palette_tables: vec![],

            freme: Frame::new(),
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
                self.ppudata = self.mem_read(self.vram_addr);
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
                self.mem_write(self.oamaddr as u16, self.oamdata);
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
                self.mem_write(self.vram_addr, self.ppudata);
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

    fn mem_read(&mut self, addr: u16) -> u8 {
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

    fn mem_write(&mut self, addr: u16, data: u8) {
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

    pub fn read_palette_table(&self, scanline: usize) -> &[u8; 32]
    {
        if self.scanline_palette_indexes.is_empty() {
            return &self.pram;
        }

        let mut index = 0;
        for (i, s) in self.scanline_palette_indexes.iter().enumerate() {
            if *s > scanline {
                break;
            }
            index = i
        }
        let table = &self.scanline_palette_tables[index];
        table
    }
}


// ==========================================================================
static mut S_PPU: Lazy<Pin<Box<PPU>>> = Lazy::new(|| {
    let ppu = Box::pin(PPU::new());
    ppu
});

// ==========================================================================

fn get_reg_config()
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


pub fn render(frame: &mut Frame)
{
    unsafe {
        // draw background
        let scroll_x = (S_PPU.scroll_x) as usize;
        let scroll_y = (S_PPU.scroll_y) as usize;

        let (main_name_table, second_name_table) = match (&get_chr_rom_mirroring(), S_PPU.name_table) {
            (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) => {
                (&S_PPU.vram[0x000..0x400], &S_PPU.vram[0x400..0x800])
            }
            (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) => {
                (&S_PPU.vram[0x400..0x800], &S_PPU.vram[0x000..0x400])
            }
            (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => {
                (&S_PPU.vram[0x000..0x400], &S_PPU.vram[0x400..0x800])
            }
            (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => {
                (&S_PPU.vram[0x400..0x800], &S_PPU.vram[0x000..0x400])
            }
            (_, _) => {
                panic!("Not supported mirroring type {:?}", get_chr_rom_mirroring());
            }
        };

        // 左上
        render_name_table(
            frame,
            main_name_table,
            Rect::new(scroll_x, scroll_y, SCREEN_WIDTH, SCREEN_HEIGHT),
            -(scroll_x as isize),
            -(scroll_y as isize),
        );

        // 右下
        render_name_table(
            frame,
            second_name_table,
            Rect::new(0, 0, scroll_x, scroll_y),
            (SCREEN_WIDTH - scroll_x) as isize,
            (SCREEN_HEIGHT - scroll_y) as isize,
        );

        // 左下
        render_name_table(
            frame,
            main_name_table,
            Rect::new(scroll_x, 0, SCREEN_WIDTH, scroll_y),
            -(scroll_x as isize),
            (SCREEN_HEIGHT - scroll_y) as isize,
        );

        // 右上
        render_name_table(
            frame,
            second_name_table,
            Rect::new(0, scroll_y, scroll_x, SCREEN_HEIGHT),
            (SCREEN_WIDTH - scroll_x) as isize,
            -(scroll_y as isize),
        );

        // draw sprites
        // TODO 8x16 mode
        for i in (0..S_PPU.oam.len()).step_by(4).rev() {
            let tile_y = S_PPU.oam[i] as usize;
            let tile_idx = S_PPU.oam[i + 1] as u16;
            let attr = S_PPU.oam[i + 2];
            let tile_x = S_PPU.oam[i + 3] as usize;

            let flip_vertical = (attr >> 7 & 1) == 1;
            let flip_horizontal = (attr >> 6 & 1) == 1;
            let palette_idx = attr & 0b11;
            let sprite_palette = sprite_palette(tile_y, palette_idx);

            let bank: u16 = S_PPU.sprite_pattern_tbl;

            let tile: &[u8] = get_chr_rom_ptr((bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize);

            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];
                'ololo: for x in (0..=7).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => continue 'ololo, // skip coloring the pixel
                        1 => COLOR_PALLETE[sprite_palette[1] as usize],
                        2 => COLOR_PALLETE[sprite_palette[2] as usize],
                        3 => COLOR_PALLETE[sprite_palette[3] as usize],
                        _ => panic!("can't be"),
                    };

                    match (flip_horizontal, flip_vertical) {
                        (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                        (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                        (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                        (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                    }
                }
            }
        }
    }
}

fn bg_pallette(attribute_table: &[u8], tile_column: usize, tile_row: usize,) -> [u8; 4]
{
    unsafe {
        let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
        let attr_byte = attribute_table[attr_table_idx];

        let pallet_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (1, 0) => (attr_byte >> 2) & 0b11,
            (0, 1) => (attr_byte >> 4) & 0b11,
            (1, 1) => (attr_byte >> 6) & 0b11,
            _ => panic!("should not happen"),
        };

        let pallette_start: usize = 1 + (pallet_idx as usize) * 4;
        let p = S_PPU.read_palette_table(tile_row * 8);
        [
            p[0],
            p[pallette_start],
            p[pallette_start + 1],
            p[pallette_start + 2],
        ]
    }
}

fn sprite_palette(tile_y: usize, palette_idx: u8) -> [u8; 4]
{
    unsafe {
        let start = 0x11 + (palette_idx * 4) as usize;
        let p = S_PPU.read_palette_table(tile_y);
        [0, p[start], p[start + 1], p[start + 2]]
    }
}

fn render_name_table(frame: &mut Frame, name_table: &[u8], view_port: Rect, shift_x: isize, shift_y: isize,)
{
    unsafe {

        let bank = S_PPU.bg_pattern_tbl;
        let attribute_table = &name_table[0x03C0..0x0400];

        for i in 0..0x03C0 {
            let tile_column = i % 32;
            let tile_row = i / 32;
            let tile_idx = name_table[i] as u16;
            let tile: &[u8] =
                get_chr_rom_ptr((bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize);
            let palette = bg_pallette(attribute_table, tile_column, tile_row);

            for y in 0..=7 {
                let mut upper = tile[y];
                let mut lower = tile[y + 8];

                for x in (0..=7).rev() {
                    let value = (1 & lower) << 1 | (1 & upper);
                    upper = upper >> 1;
                    lower = lower >> 1;
                    let rgb = match value {
                        0 => COLOR_PALLETE[palette[0] as usize],
                        1 => COLOR_PALLETE[palette[1] as usize],
                        2 => COLOR_PALLETE[palette[2] as usize],
                        3 => COLOR_PALLETE[palette[3] as usize],
                        _ => panic!("can't be"),
                    };

                    let pixel_x = tile_column * 8 + x;
                    let pixel_y = tile_row * 8 + y;
                    if pixel_x >= view_port.x1
                        && pixel_x < view_port.x2
                        && pixel_y >= view_port.y1
                        && pixel_y < view_port.y2
                    {
                        frame.set_pixel(
                            (shift_x + pixel_x as isize) as usize,
                            (shift_y + pixel_y as isize) as usize,
                            rgb,
                        )
                    }
                }
            }
        }
    }
}

fn nmi_gen()
{
    cpu_interrupt(InterruptType::NMI);
    print!("[DEBUG]: PPU V-Blank! NMI Generated!");
}

static mut CANVAS: Option<Canvas<Window>> = None;
static mut TEXTURE: Option<Box<Texture<'static>>> = None;
static mut TEXTURE_CREATOR: Option<*mut TextureCreator<WindowContext>> = None;

fn display_render() {
    unsafe {
        if let Some(canvas) = CANVAS.as_mut() {
            if let Some(texture) = TEXTURE.as_mut() {
                // 最初の240本(0〜240) で画面が描画
                // data_set(S_PPU.scanline);
                render(&mut S_PPU.freme);

                // SDL2
                texture.update(None, &mut S_PPU.freme.data, 256 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }
    }
}

fn sdl2_init(sdl_context: Sdl) {
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rust NES Emulator", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let creator: *mut TextureCreator<WindowContext> = Box::into_raw(Box::new(canvas.texture_creator()));
    let texture = Box::new(
        unsafe {
            (*creator).create_texture_target(PixelFormatEnum::RGB24, 256, 240).unwrap()
        },
    );

    unsafe {
        CANVAS = Some(canvas);
        TEXTURE = Some(texture);
        TEXTURE_CREATOR = Some(creator);
    }
}

pub fn ppu_reset(sdl_context: Sdl) -> Box<PPU>
{
    unsafe {
        let ppu_box: Box<PPU> = Box::from_raw(Pin::as_mut(&mut *S_PPU).get_mut());

        // SDL2 グラフィック関連初期化
        sdl2_init(sdl_context);

        S_PPU.cycle = 0;
        S_PPU.scanline = 0;

        // V-Blak開始
        S_PPU.ppustatus |= REG_PPUSTATUS_BIT_VBLANK;
        ppu_box
    }
}

// NTSC 60FPS（59.94FPS）のPPU処理
// ※1フレーム = 1/60fps(1/59.94) ≒ 16.668 msec
pub fn ppu_main()
{
    unsafe {
        if S_PPU.cycle == 0 {
            S_PPU.oamdma_done = false;
            get_reg_config(); // レジスタ吸出し
        }

        // [240ライン描画]
        // (240 x 341クロック) = 81,840クロック
        // ≒ 15,242,160.6656712 nsec ≒ 15.242 msec
        // ※T ≒ 186.2434098933431 nsec
        if S_PPU.scanline < 241 {
            display_render(); // 画面描画(@SDL2)
        }

        // [V-Blank開始]
        // 残22本分(241〜262) -> 垂直回帰時間
        // = (22 x 341クロック) = 7502クロック ≒ 1,397,198.061 nsec ≒ 1.397 msec
        // ※T ≒ 186.2434098933431 nsec
        // 垂直回帰時間のCPUクロック = 2,500.667 CPUサイクル?
        // ※1 CPU Clock = 558.7302296800292 nsec
        if S_PPU.scanline == 241 {
            S_PPU.ppustatus |= REG_PPUSTATUS_BIT_VBLANK;
            if S_PPU.nmi_gen != false {
                nmi_gen(); // NMI
            }
        }

        S_PPU.cycle += 1;
        if (S_PPU.cycle % 341) == 0 {
            S_PPU.scanline += 1;
        }

        // [V-Blank終了]
        if S_PPU.scanline >= 262  {
            S_PPU.ppustatus &= !REG_PPUSTATUS_BIT_VBLANK;
            S_PPU.cycle = 0;
            S_PPU.scanline = 0;
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