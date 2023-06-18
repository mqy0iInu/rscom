use crate::cpu::*;
use crate::ppu::*;
use crate::apu::*;
use crate::cassette::*;

pub const BIN_BIT_7: u8 = 0x80;            // bit7
pub const BIN_BIT_6: u8 = 0x40;            // bit6
pub const BIN_BIT_5: u8 = 0x20;            // bit5
pub const BIN_BIT_4: u8 = 0x10;            // bit4
pub const BIN_BIT_3: u8 = 0x08;            // bit3
pub const BIN_BIT_2: u8 = 0x04;            // bit2
pub const BIN_BIT_1: u8 = 0x02;            // bit1
pub const BIN_BIT_0: u8 = 0x01;            // bit0

pub const ADDR_CHR_ROM: u16 = 0x4020;   // CHR-ROM TOP
pub const ADDR_PRG_RAM: u16 = 0xFFFE;   // PRG-RAM TOP
pub const ADDR_PRG_ROM: u16 = 0x8000;   // PRG-ROM TOP

// const VRAM_SIZE: u16 = 0x4000;          // VRAM (16KB)
// const VRAM_START_ADDR: u16 = 0x2008;    // VRAM 開始アドレス
// const VRAM_END_ADDR: u16 = 0x3FFF;

// (DEBUG) :Snake Game(Only 6502 OP-Code)
// https://bugzmanov.github.io/nes_ebook/chapter_3_4.html
// アセンブラ ... https://gist.github.com/wkjagt/9043907
pub const SNAKE_GAME_TBL: [u8; 310] = [
    0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
    0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
    0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
    0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
    0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
    0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
    0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
    0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
    0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
    0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
    0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
    0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
    0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
    0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
    0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
    0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
    0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
    0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
    0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
    0xea, 0xca, 0xd0, 0xfb, 0x60, 0x60
];

#[derive(Clone)]
pub struct Memory {
    pub wram: [u8; 2048],         // WRAM ... 2KB (For RP2A03)
    pub vram: [u8; 2048],         // VRAM ... 2KB (For PPU)
    pub dma_start_addr: u8,
    pub apu_reg: APUReg,          // APUレジスタ
    pub ppu_reg: PPU,          // PPUレジスタ
    pub cassette: Cassette,       // カセット
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            wram: [0; 2048],
            vram: [0; 2048],
            dma_start_addr: 0,
            apu_reg: APUReg::new(),
            ppu_reg: PPU::new(),
            cassette: Cassette::new(),
        }
    }

    pub fn mem_reset(&mut self)
    {
        // TODO :MEM Reset
        // rom_loader(&mut self.cassette, "test_rom/nes/mapper_0/BombSweeper.nes");

        // (DEBUG) :Snake Game(Only 6502 OP-Code)
        let start_address = 0x600; // WRAMで実行する
        let end_address = start_address + SNAKE_GAME_TBL.len();
        self.wram[start_address..end_address].copy_from_slice(&SNAKE_GAME_TBL);
        self.mem_write(ADDR_VEC_TBL_RST, 0x00);
        self.mem_write(ADDR_VEC_TBL_RST + 1, 0x06);
        // =================================
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07FF => self.wram[addr as usize],
            0x0800..=0x1FFF => self.wram[(addr % 0x0800) as usize],
            0x2000..=0x2007 => self.ppu_reg.ppu_reg_ctrl(addr, PPU_REG_READ, 0),
            0x2008..=0x3FFF => self.vram[(addr - 0x2000) as usize],
            0x4000..=0x4017 => self.apu_reg.apu_reg_ctrl(addr, APU_REG_READ, 0),
            0x4020..=0x5FFF => self.cassette.chr_rom[(addr - 0x4020) as usize],
            // TODO :(DEBUG) PRG-RAM
            // 0x6000..=0x7FFF => self.cassette.chr_ram[(addr - 0x6000) as usize],
            // TODO :(DEBUG)一旦、PRG-ROMをミラーしとく
            0x8000..=0xBFFF => self.cassette.prg_rom[(addr - 0x8000) as usize],
            0xC000..=0xFFFF => self.cassette.prg_rom[(addr - 0xC000) as usize],
            _ => panic!("Invalid Mem Addr: {:#06x}", addr),
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x07FF => self.wram[addr as usize] = data,
            0x0800..=0x1FFF => self.wram[(addr % 0x0800) as usize] = data,
            0x2000..=0x2007 => { self.ppu_reg.ppu_reg_ctrl(addr, PPU_REG_WRITE, data);},
            0x2008..=0x3FFF => self.vram[(addr - 0x2000) as usize] = data,
            0x4000..=0x4013 | 0x4015 | 0x4017 => { self.apu_reg.apu_reg_ctrl(addr, APU_REG_WRITE, data);},
            0x4014          => {
                self.dma_start_addr = (addr >> 8) as u8 ;
                self.dma_start();
            },
            0x4020..=0x5FFF => self.cassette.chr_rom[(addr - 0x4020) as usize] = data,       // CHR ROM ... 8KB or 16KB
            // 0x6000..=0x7FFF => self.cassette.chr_ram[(addr - 0x6000) as usize] = data,       // Ext RAM
            0x8000..=0xBFFF => self.cassette.prg_rom[(addr - 0x8000) as usize] = data,       // PRG ROM ... 8KB ~ 1MB
            // TODO :(DEBUG)一旦、PRG-ROMをミラーしとく
            0xC000..=0xFFFF => self.cassette.prg_rom[(addr - 0xC000) as usize] = data,       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid Mem Addr: {:#06x}", addr),
        }
    }

    pub fn dma_start(&mut self)
    {
        let mut start_addr:u16 = self.dma_start_addr as u16;
        start_addr =  (start_addr << 0x08u8) as u16;
        println!("[DEBUG] : DMA Start");

        cpu_run(false);
        // WRAM to OAM (256Byte)
        for i in 0..=255 {
            let mut data = self.mem_read(start_addr);
            self.ppu_reg.ppu_oam_write(i, data);
            start_addr = start_addr + 1;
        }
        cpu_run(true);
    }
}


// ====================================== TEST ======================================
#[cfg(test)]
mod mem_test {

    #[test]
    fn mem_test() {
        // TODO : MEM Test
    }
}
// ==================================================================================