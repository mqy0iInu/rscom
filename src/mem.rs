use crate::cpu::*;
use crate::ppu::*;
use crate::apu::*;

pub const BIN_BIT_7: u8 = 0x80;                     // bit7
pub const BIN_BIT_6: u8 = 0x40;                     // bit6
pub const BIN_BIT_5: u8 = 0x20;                     // bit5
pub const BIN_BIT_4: u8 = 0x10;                     // bit4
pub const BIN_BIT_3: u8 = 0x08;                     // bit3
pub const BIN_BIT_2: u8 = 0x04;                     // bit2
pub const BIN_BIT_1: u8 = 0x02;                     // bit1
pub const BIN_BIT_0: u8 = 0x01;                     // bit0

pub const ADDR_CHR_ROM: u16 = 0x4020;               // CHR-ROM TOP
pub const ADDR_PRG_RAM: u16 = 0xFFFE;               // PRG-RAM TOP
pub const ADDR_PRG_ROM: u16 = 0x8000;               // PRG-ROM TOP
pub const ADDR_VEC_TBL_RST: u16 = 0xFFFC;           // RESET Vector Table
pub const ADDR_VEC_TBL_IRQ: u16 = 0xFFFE;           // IRQ Vector Table
pub const ADDR_VEC_TBL_NMI: u16 = 0xFFFA;           // NMI Vector Table
const VRAM_SIZE: u16 = 0x4000;                      // VRAM (16KB)
const VRAM_START_ADDR: u16 = 0x2008;                // VRAM 開始アドレス
const VRAM_END_ADDR: u16 = 0x3FFF;

pub const DMA_START_ADDR:u16 = 0x4014;
pub const DMA_SIZE:u16 = 0x4014;

pub struct NESMemory {
    pub wram: [u8; 2048],         // WRAM ... 2KB (For RP2A03)
    pub vram: [u8; 2048],         // VRAM ... 2KB (For PPU)
    pub dma_start_addr: u8,
    pub apu_reg: APUReg,          // APUレジスタ
    pub ppu_reg: PPUReg,          // PPUレジスタ
    pub chr_rom: Vec<u8>,         // CHR ROM ... 8KB or 16KB
    pub ext_ram: Vec<u8>,         // Ext RAM
    pub prg_rom: Vec<u8>,         // PRG ROM ... 8KB ~ 1MB
}

impl NESMemory {
    pub fn new() -> Self {
        NESMemory {
            wram: [0; 2048],
            vram: [0; 2048],
            dma_start_addr: 0,
            apu_reg: APUReg::new(),
            ppu_reg: PPUReg::new(),
            chr_rom: Vec::new(),
            ext_ram: Vec::new(),
            prg_rom: Vec::new(),
        }
    }

    pub fn mem_reset(&mut self)
    {
        // TODO :MEM Reset

        // (DEBUG) :ダミーROMデータ
        // ROM = $8000~$8015でロード、ストア、演算命令をループ
        self.prg_rom.extend([0x38, 0xF8, 0x78, 0x18, 0xD8, 0x58, 0xB8].iter().cloned());
        self.prg_rom.extend([0xA9, 0x0A, 0xAA, 0x8A, 0xA9, 0x0B, 0xA8, 0x98].iter().cloned());
        self.prg_rom.extend([0x09, 0xA0, 0x49, 0xBA, 0x29, 0x44].iter().cloned());
        self.prg_rom.extend([0x4C, 0x00, 0x80].iter().cloned());
    }

    pub fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07FF => self.wram[addr as usize],                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(addr % 0x0800) as usize],          // RAMのミラーリング
            0x2000..=0x2007 => self.ppu_reg.ppu_reg_ctrl(addr, PPU_REG_READ, 0), // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(addr - 0x2000) as usize],          // VRAM ... 2KB (For PPU)
            0x4000..=0x4017 => self.apu_reg.apu_reg_ctrl(addr, APU_REG_READ, 0), // APUレジスタ
            0x4020..=0x5FFF => self.chr_rom[(addr - 0x4020) as usize],       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.ext_ram[(addr - 0x6000) as usize],       // Ext RAM
            0x8000..=0xFFFF => self.prg_rom[(addr - 0x8000) as usize],       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid Mem Addr: {:#06x}", addr),
        }
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x07FF => self.wram[addr as usize] = data,                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(addr % 0x0800) as usize] = data,          // RAMのミラーリング
            0x2000..=0x2007 => { self.ppu_reg.ppu_reg_ctrl(addr, PPU_REG_WRITE, data);}, // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(addr - 0x2000) as usize] = data,          // VRAM ... 2KB (For PPU)
            0x4000..=0x4013 | 0x4015 | 0x4017 => { self.apu_reg.apu_reg_ctrl(addr, APU_REG_WRITE, data);}, // APUレジスタ
            0x4014          => {
                self.dma_start_addr = (addr >> 8) as u8 ;
                self.dma_start();
            },
            0x4020..=0x5FFF => self.chr_rom[(addr - 0x4020) as usize] = data,       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.ext_ram[(addr - 0x6000) as usize] = data,       // Ext RAM
            0x8000..=0xFFFF => self.prg_rom[(addr - 0x8000) as usize] = data,       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid Mem Addr: {:#06x}", addr),
        }
    }
    pub fn dma_start(&mut self)
    {
        let mut start_addr:u16 = self.dma_start_addr as u16;
        start_addr =  (start_addr << 0x08u8) as u16;
        println!("[DEBUG] : DMA Start");

        cpu_stop(true);
        // WRAM to OAM (256Byte)
        for i in 0..=255 {
            let mut data = self.mem_read(start_addr);
            self.ppu_reg.ppu_oam_write(i, data);
            start_addr = start_addr + 1;
        }
        cpu_stop(false);
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