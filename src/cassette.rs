use std::path::Path;
use std::fs::File;
use std::io::Read;

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

pub const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // ASCII "NES" + EOF(0x1A)
pub const PRG_ROM_SIZE: usize = 16 * 1024;             // 16KB
// pub const PRG_ROM_SIZE: usize = 32 * 1024;          // 32KB
pub const CHR_ROM_SIZE: usize = 8 * 1024;              // 8KB

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        let prg_rom_size = raw[4] as usize * PRG_ROM_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_SIZE;

        // iNES Check
        if &raw[0..4] != NES_TAG {
            panic!("[ERR]: Not iNES Format ROM !!!");
        }
        // (DEBUG)
        if raw.len() < 16 + prg_rom_size + chr_rom_size {
            panic!("[ERR]: ROM size is larger than the Emu buffer size!!!");
        }

        let mapper = (raw[6] >> 0x04) | (raw[7] & 0xF0);
        let four_screen_mirroring  = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring =
            match (four_screen_mirroring , vertical_mirroring) {
                (true, _) => Mirroring::FOUR_SCREEN,
                (false, true) => Mirroring::VERTICAL,
                (false, false) => Mirroring::HORIZONTAL,
                (_, _) => panic!("[ERR]: But Rom iNES !!!"),
            };

        let skip_trainer = raw[6] & 0b100 != 0;
        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        // (DEBUG)iNES Header
        let ines_header = &raw[0..16];
        println!("[DEBUG]: ROM Read! iNES Header Dump!!!");
        for byte in ines_header {
            print!("[{:#04x}]", byte);
        }
        println!("\n");

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        })
    }

    pub fn rom_clear() -> Self {
        return Rom {
            prg_rom: vec![0; PRG_ROM_SIZE],
            chr_rom: vec![0; CHR_ROM_SIZE],
            mapper: 0,
            screen_mirroring: Mirroring::VERTICAL,
        };
    }
}

pub fn rom_loader(path :&str) -> Rom
{
    let mut f = File::open(path).expect("[ERR]: No File Found!!!");
    let metadata = std::fs::metadata(path).expect("[ERR]: Unable Read Metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("[ERR]: File Read Buffer OVF");
    let rom = Rom::new(&buffer).expect("[ERR]: ROM Read ERROR");
    rom
}

// ROM info Dump
pub fn rom_info_dump(path:&str, rom :&Rom)
{
    let filename = Path::new(path)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("");

    println!("*********** [NES ROM Info] ***********");
    println!("NES File: {}", filename);
    println!("PRG-ROM Size: {} bytes", rom.prg_rom.len());
    println!("CHR-ROM Size: {} bytes", rom.chr_rom.len());
    println!("Mapper: {}", rom.mapper);
    println!("Screen Mirroring: {:X?}", rom.screen_mirroring);
    println!("****************************************\n");
}
// ====================================== TEST ======================================
mod rom_test {
    use super::*;

    // [Mapper 0]
    const ROM_MAPPER_0_ROM_TBL: [&str; 13] = [
        "test_rom/nes/mapper_0/1942.nes",
        "test_rom/nes/mapper_0/donkeykong.nes",
        "test_rom/nes/mapper_0/elevatoraction.nes",
        "test_rom/nes/mapper_0/excitebike.nes",
        "test_rom/nes/mapper_0/galaga.nes",
        "test_rom/nes/mapper_0/galaxian.nes",
        "test_rom/nes/mapper_0/ikki.nes",
        "test_rom/nes/mapper_0/karateka.nes",
        "test_rom/nes/mapper_0/mappy.nes",
        "test_rom/nes/mapper_0/pacman.nes",
        "test_rom/nes/mapper_0/super_mario_bros.nes",
        "test_rom/nes/mapper_0/tower_of_druaga.nes",
        "test_rom/nes/mapper_0/xevious.nes",
    ];

    #[test]
    fn rom_test()
    {
        for path in ROM_MAPPER_0_ROM_TBL.iter() {
            let rom = rom_loader(path);
            rom_info_dump(&path, &rom);
        }
    }
}
// ==================================================================================