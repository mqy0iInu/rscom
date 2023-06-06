use crate::ppu::*;
use crate::apu::*;

pub const ADDR_CHR_ROM: u16 = 0x4020;               // CHR-ROM TOP
pub const ADDR_PRG_RAM: u16 = 0xFFFE;               // PRG-RAM TOP
pub const ADDR_PRG_ROM: u16 = 0x8000;               // PRG-ROM TOP
pub const ADDR_VEC_TBL_RST: u16 = 0xFFFC;           // RESET Vector Table
pub const ADDR_VEC_TBL_IRQ: u16 = 0xFFFE;           // IRQ Vector Table
pub const ADDR_VEC_TBL_NMI: u16 = 0xFFFA;           // NMI Vector Table

pub struct NESMemory {
    wram: [u8; 2048],         // WRAM ... 2KB (For RP2A03)
    vram: [u8; 2048],         // VRAM ... 2KB (For PPU)
    apu_reg: APUReg,          // APUレジスタ
    ppu_reg: PPUReg,          // PPUレジスタ
    chr_rom: Vec<u8>,         // CHR ROM ... 8KB or 16KB
    ext_ram: Vec<u8>,         // Ext RAM
    prg_rom: Vec<u8>,         // PRG ROM ... 8KB ~ 1MB
}

impl NESMemory {
    pub fn new() -> Self {
        NESMemory {
            wram: [0; 2048],
            vram: [0; 2048],
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

    pub fn mem_read(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x07FF => self.wram[address as usize],                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(address % 0x0800) as usize],          // RAMのミラーリング
            0x2000..=0x2007 => self.ppu_reg.ppu_reg_ctrl(address, PPU_REG_READ, 0), // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(address - 0x2000) as usize],          // VRAM ... 2KB (For PPU)
            0x4000..=0x4017 => self.apu_reg.apu_reg_ctrl(address, APU_REG_READ, 0), // APUレジスタ
            0x4020..=0x5FFF => self.chr_rom[(address - 0x4020) as usize],       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.ext_ram[(address - 0x6000) as usize],       // Ext RAM
            0x8000..=0xFFFF => self.prg_rom[(address - 0x8000) as usize],       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid memory address: {:#06x}", address),
        }
    }

    pub fn mem_write(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x07FF => self.wram[address as usize] = data,                     // WRAM ... 2KB (For RP2A03)
            0x0800..=0x1FFF => self.wram[(address % 0x0800) as usize] = data,          // RAMのミラーリング
            0x2000..=0x2007 => { self.ppu_reg.ppu_reg_ctrl(address, PPU_REG_WRITE, data);}, // PPUレジスタ
            0x2008..=0x3FFF => self.vram[(address - 0x2000) as usize] = data,          // VRAM ... 2KB (For PPU)
            0x4000..=0x4017 => { self.apu_reg.apu_reg_ctrl(address, APU_REG_WRITE, data);}, // APUレジスタ
            0x4020..=0x5FFF => self.chr_rom[(address - 0x4020) as usize] = data,       // CHR ROM ... 8KB or 16KB
            0x6000..=0x7FFF => self.ext_ram[(address - 0x6000) as usize] = data,       // Ext RAM
            0x8000..=0xFFFF => self.prg_rom[(address - 0x8000) as usize] = data,       // PRG ROM ... 8KB ~ 1MB
            _ => panic!("Invalid memory address: {:#06x}", address),
        }
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