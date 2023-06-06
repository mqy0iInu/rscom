// ==================================================================================
// PPU Register
const PPU_CTRL_1_REG_ADDR: u16 = 0x2000;
const PPU_CTRL_2_REG_ADDR: u16 = 0x2001;
const PPU_STATUS_REG_ADDR: u16 = 0x2002;
const PPU_SPRITE_MEM_ADDR_REG_ADDR: u16 = 0x2003;
const PPU_SPRITE_MEM_DATA_REG_ADDR: u16 = 0x2004;
const PPU_BG_SCROLL_REG_ADDR: u16 = 0x2005;
const PPU_PPU_MEM_ADDR_REG_ADDR: u16 = 0x2006;
const PPU_PPU_MEM_DATA_REG_ADDR: u16 = 0x2007;

// ==================================================================================
// PPU Memory
// const PPU_OAM_SIZE: usize = 0x0100;  // OAM（Object Attribute Memory）のサイズ (256バイト)
// const PPU_OAM_START_ADDR: u16 = 0x0000;  // OAMの開始アドレス
// const PPU_OAM_END_ADDR: u16 = PPU_OAM_START_ADDR + PPU_OAM_SIZE as u16 - 1;  // OAMの終了アドレス

// ==================================================================================
// PPU Memory Map
// const PPU_PATTERN_TABLE_0_START_ADDR: u16 = 0x0000;  // パターンテーブル#0の開始アドレス
// const PPU_PATTERN_TABLE_0_END_ADDR: u16 = 0x0FFF;  // パターンテーブル#0の終了アドレス

// const PPU_PATTERN_TABLE_1_START_ADDR: u16 = 0x1000;  // パターンテーブル#1の開始アドレス
// const PPU_PATTERN_TABLE_1_END_ADDR: u16 = 0x1FFF;  // パターンテーブル#1の終了アドレス

// const PPU_NAME_TABLE_0_START_ADDR: u16 = 0x2000;  // ネームテーブル#0の開始アドレス
// const PPU_NAME_TABLE_0_END_ADDR: u16 = 0x23BF;  // ネームテーブル#0の終了アドレス

// const PPU_ATTRIBUTE_TABLE_0_START_ADDR: u16 = 0x23C0;
// const PPU_ATTRIBUTE_TABLE_0_START_END: u16 = 0x23FF;

// const PPU_NAME_TABLE_1_START_ADDR: u16 = 0x2000;  // ネームテーブル#1の開始アドレス
// const PPU_NAME_TABLE_1_END_ADDR: u16 = 0x23BF;  // ネームテーブル#1の終了アドレス

// const PPU_ATTRIBUTE_TABLE_1_START_ADDR: u16 = 0x23C0;
// const PPU_ATTRIBUTE_TABLE_1_START_END: u16 = 0x23FF;

// const PPU_NAME_TABLE_2_START_ADDR: u16 = 0x2800;  // ネームテーブル#1の開始アドレス
// const PPU_NAME_TABLE_2_END_ADDR: u16 = 0x2BBF;  // ネームテーブル#1の終了アドレス

// const PPU_ATTRIBUTE_TABLE_2_START_ADDR: u16 = 0x2BC0;
// const PPU_ATTRIBUTE_TABLE_2_START_END: u16 = 0x2BFF;

// const PPU_NAME_TABLE_3_START_ADDR: u16 = 0x2C00;  // ネームテーブル#1の開始アドレス
// const PPU_NAME_TABLE_3_END_ADDR: u16 = 0x2FBF;  // ネームテーブル#1の終了アドレス

// const PPU_ATTRIBUTE_TABLE_3_START_ADDR: u16 = 0x2FC0;
// const PPU_ATTRIBUTE_TABLE_3_START_END: u16 = 0x2FFF;

// const PPU_PALETTE_START_ADDR: u16 = 0x3F00;  // BGパレットの開始アドレス
// const PPU_PALETTE_END_ADDR: u16 = 0x3F0F;
// const PPU_PALETTE_SIZE: u16 = 0x0010;  // BGパレットのサイズ

// const PPU_PATTERN_TABLE_SIZE: u16 = 0x1000;
// const PPU_NAME_TABLE_SIZE: u16 = 0x03C0;
// const PPU_ATTRIBUTE_TABLE_SIZE: u16 = 0x0040;
// ==================================================================================
pub const PPU_REG_READ: u8 = 0x00;
pub const PPU_REG_WRITE: u8 = 0x01;

pub struct PPUReg {
    ctrl_1_reg: u8,          // ($2000) (RW) Control Register 1
    ctrl_2_reg: u8,          // ($2001) (RW) Control Register 2
    status_reg: u8,          // ($2002) (RW) Status Register
    sprite_mem_addr_reg: u8, // ($2003) (RW) Sprite Memory Address Register
    sprite_mem_data_reg: u8, // ($2004) (RW) Sprite Memory Data Register
    bg_scroll_reg: u8,       // ($2005) (RW) BG Scroll Register
    ppu_mem_addr_reg: u8,    // ($2006) (RW) PPU Mem, Addr Register
    ppu_mem_data_reg: u8,    // ($2007) (RW) PPU Mem, Data Register
    oam: [u8; 0x100],        // OAM
}

impl PPUReg {
    pub fn new() -> Self {
        PPUReg {
            ctrl_1_reg: 0,          // ($2000) (RW) Control Register 1
            ctrl_2_reg: 0,          // ($2001) (RW) Control Register 2
            status_reg: 0,          // ($2002) (RW) Status Register
            sprite_mem_addr_reg: 0, // ($2003) (RW) Sprite Memory Address Register
            sprite_mem_data_reg: 0, // ($2004) (RW) Sprite Memory Data Register
            bg_scroll_reg: 0,       // ($2005) (RW) BG Scroll Register
            ppu_mem_addr_reg: 0,    // ($2006) (RW) PPU Mem, Addr Register
            ppu_mem_data_reg: 0,    // ($2007) (RW) PPU Mem, Data Register
            oam: [0; 0x100],
        }
    }

    fn ppu_reg_read(&self, address: u16) -> u8
    {
        match  address {
            PPU_CTRL_1_REG_ADDR => self.ctrl_1_reg,
            PPU_CTRL_2_REG_ADDR => self.ctrl_2_reg,
            PPU_STATUS_REG_ADDR => self.status_reg,
            PPU_SPRITE_MEM_ADDR_REG_ADDR => self.sprite_mem_addr_reg,
            PPU_SPRITE_MEM_DATA_REG_ADDR => self.sprite_mem_data_reg,
            PPU_BG_SCROLL_REG_ADDR => self.bg_scroll_reg,
            PPU_PPU_MEM_ADDR_REG_ADDR => self.ppu_mem_addr_reg,
            PPU_PPU_MEM_DATA_REG_ADDR => self.ppu_mem_data_reg,
            _ => panic!("Invalid PPU Register Address: 0x{:04X}", address),
        }
    }

    fn ppu_reg_write(&mut self, address: u16, data: u8)
    {
        match  address {
            PPU_CTRL_1_REG_ADDR => self.ctrl_1_reg = data,
            PPU_CTRL_2_REG_ADDR => self.ctrl_2_reg = data,
            PPU_STATUS_REG_ADDR => self.status_reg = data,
            PPU_SPRITE_MEM_ADDR_REG_ADDR => self.sprite_mem_addr_reg = data,
            PPU_SPRITE_MEM_DATA_REG_ADDR => self.sprite_mem_data_reg = data,
            PPU_BG_SCROLL_REG_ADDR => self.bg_scroll_reg = data,
            PPU_PPU_MEM_ADDR_REG_ADDR => self.ppu_mem_addr_reg = data,
            PPU_PPU_MEM_DATA_REG_ADDR => self.ppu_mem_data_reg = data,
            _ => panic!("Invalid PPU Register Address: 0x{:04X}", address),
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

    pub fn ppu_oam_read(&mut self, addr: u8) -> u8
    {
        self.oam[addr as usize]
    }

    pub fn ppu_oam_write(&mut self, addr: u8, data: u8)
    {
        self.oam[addr as usize] = data;
    }
}

pub fn ppu_reset()
{
    // TODO :PPU Init
}

pub fn ppu_main()
{
    println!("[DEBUG] : PPU Main Loop");
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