// =========================================================================
// [Common Define]
// =========================================================================
pub const _BIT_0:   u8 = 0x00000001;
pub const _BIT_1:   u8 = 0x00000002;
pub const _BIT_2:   u8 = 0x00000004;
pub const _BIT_3:   u8 = 0x00000008;
pub const _BIT_4:   u8 = 0x00000010;
pub const _BIT_5:   u8 = 0x00000020;
pub const _BIT_6:   u8 = 0x00000040;
pub const _BIT_7:   u8 = 0x00000080;
pub const _BIT_8:  u16 = 0x00000100;
pub const _BIT_9:  u16 = 0x00000200;
pub const _BIT_10: u16 = 0x00000400;
pub const _BIT_11: u16 = 0x00000800;
pub const _BIT_12: u16 = 0x00001000;
pub const _BIT_13: u16 = 0x00002000;
pub const _BIT_14: u16 = 0x00004000;
pub const _BIT_15: u16 = 0x00008000;
pub const _BIT_16: u32 = 0x00010000;
pub const _BIT_17: u32 = 0x00020000;
pub const _BIT_18: u32 = 0x00040000;
pub const _BIT_19: u32 = 0x00080000;
pub const _BIT_20: u32 = 0x00100000;
pub const _BIT_21: u32 = 0x00200000;
pub const _BIT_22: u32 = 0x00400000;
pub const _BIT_23: u32 = 0x00800000;
pub const _BIT_24: u32 = 0x01000000;
pub const _BIT_25: u32 = 0x02000000;
pub const _BIT_26: u32 = 0x04000000;
pub const _BIT_27: u32 = 0x08000000;
pub const _BIT_28: u32 = 0x10000000;
pub const _BIT_29: u32 = 0x20000000;
pub const _BIT_30: u32 = 0x40000000;
pub const _BIT_31: u32 = 0x80000000;

pub const _ADDR_A_0:  u8 =  0;
pub const _ADDR_A_1:  u8 =  1;
pub const _ADDR_A_2:  u8 =  2;
pub const _ADDR_A_3:  u8 =  3;
pub const _ADDR_A_4:  u8 =  4;
pub const _ADDR_A_5:  u8 =  5;
pub const _ADDR_A_6:  u8 =  6;
pub const _ADDR_A_7:  u8 =  7;
pub const _ADDR_A_8:  u8 =  8;
pub const _ADDR_A_9:  u8 =  9;
pub const _ADDR_A_10: u8 = 10;
pub const _ADDR_A_11: u8 = 11;
pub const _ADDR_A_12: u8 = 12;
pub const _ADDR_A_13: u8 = 13;
pub const _ADDR_A_14: u8 = 14;
pub const _ADDR_A_15: u8 = 15;
pub const _ADDR_A_16: u8 = 16;
pub const _ADDR_A_17: u8 = 17;
pub const _ADDR_A_18: u8 = 18;
pub const _ADDR_A_19: u8 = 19;
pub const _ADDR_A_20: u8 = 20;
pub const _ADDR_A_21: u8 = 21;
pub const _ADDR_A_22: u8 = 22;
pub const _ADDR_A_23: u8 = 23;
pub const _ADDR_A_24: u8 = 24;
pub const _ADDR_A_25: u8 = 25;
pub const _ADDR_A_26: u8 = 26;
pub const _ADDR_A_27: u8 = 27;
pub const _ADDR_A_28: u8 = 28;
pub const _ADDR_A_29: u8 = 29;
pub const _ADDR_A_30: u8 = 30;
pub const _ADDR_A_31: u8 = 31;

pub const _MEM_SIZE_4K:   u16 =   4 * 1024;
pub const _MEM_SIZE_8K:   u16 =   8 * 1024;
pub const _MEM_SIZE_16K:  u16 =  16 * 1024;
pub const _MEM_SIZE_32K:  u16 =  32 * 1024;
pub const _MEM_SIZE_64K:  u32 =  64 * 1024;
pub const _MEM_SIZE_128K: u32 = 128 * 1024;
pub const _MEM_SIZE_256K: u32 = 256 * 1024;
pub const _MEM_SIZE_512K: u32 = 512 * 1024;

pub const _MMC_0: u8 = 0;
pub const _MMC_1: u8 = 1;
pub const _MMC_2: u8 = 2;
pub const _MMC_3: u8 = 3;
pub const _MMC_4: u8 = 4;

pub const _MAPPER_0: u8 = 0;
pub const _MAPPER_1: u8 = 1;
pub const _MAPPER_2: u8 = 2;
pub const _MAPPER_3: u8 = 3;
pub const _MAPPER_4: u8 = 4;
pub const _MAPPER_105: u8 = 105;
pub const _MAPPER_115: u8 = 115;
pub const _MAPPER_118: u8 = 118;
pub const _MAPPER_119: u8 = 119;

pub const _CHR_ROM: u8 = 0;
pub const _CHR_RAM: u8 = 1;
pub const _PRG_ROM: u8 = 2;
// =========================================================================
// [動作OK]
// =========================================================================
// [Mapper0]
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/Alter_Ego.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/BombSweeper.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/donkeykong.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/elevatoraction.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/excitebike.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/galaga.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/mario_bros.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/pacman.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/popeye.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/ikki.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/sky_destroyer.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/Super_Mario_Bros.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/tower_of_druaga.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/xevious.nes";

// [MapperMMC 2]
pub const _NES_ROM_PATH: &str = "rom/nes/Dragon Quest 2 (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/Makaimura (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/Rockman (J).nes";

// [MapperMMC 3]
// pub const _NES_ROM_PATH: &str = "rom/nes/Dragon Quest.nes";

// =========================================================================
// [動作NG]
// =========================================================================
// [Mapper 0]
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/golf.nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/mapper_0/ice_climber.nes";

// [MapperMMC 1]
// pub const _NES_ROM_PATH: &str = "rom/nes/Dragon Quest 3 (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/Dragon Quest 4 (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/zelda.nes";

// [MapperMMC 3]
// pub const _NES_ROM_PATH: &str = "rom/nes/soromon_no_kagi.nes";

// [Mapper 4]
// pub const _NES_ROM_PATH: &str = "rom/nes/Hoshi no Kirby (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/Super Mario Bros 3  (J).nes";
// pub const _NES_ROM_PATH: &str = "rom/nes/Final Fantasy 3  (J).nes";

// [Mapper 184]
// pub const _NES_ROM_PATH: &str = "rom/nes/Atlantis no Nazo (J).nes";