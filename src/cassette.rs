use std::path::Path;
use std::fs::File;
use std::io::Read;

// [Mapper 0]
pub const ROM_MAPPER_0_ROM_TBL: [&str; 14] = [
    "test_rom/nes/mapper_0/1942.nes",
    "test_rom/nes/mapper_0/donkeykong.nes",
    "test_rom/nes/mapper_0/elevatoraction.nes",
    "test_rom/nes/mapper_0/excitebike.nes",
    "test_rom/nes/mapper_0/galaga.nes",
    "test_rom/nes/mapper_0/galaxian.nes",
    "test_rom/nes/mapper_0/ikki.nes",
    "test_rom/nes/mapper_0/karateka.nes",
    "test_rom/nes/mapper_0/mario_bros.nes",
    "test_rom/nes/mapper_0/mappy.nes",
    "test_rom/nes/mapper_0/popeye.nes",
    "test_rom/nes/mapper_0/sky_destroyer.nes",
    "test_rom/nes/mapper_0/tower_of_druaga.nes",
    "test_rom/nes/mapper_0/xevious.nes",
];

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
}

pub const NES_ASCII: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // ASCII "NES" + EOF(0x1A)
pub const PRG_ROM_MIN_SIZE: usize = 16 * 1024;           // 16KB
pub const CHR_RAM_MIN_SIZE: usize = 16 * 1024;           // 16KB(TBD)
pub const CHR_ROM_MIN_SIZE: usize = 8 * 1024;            // 8KB

pub struct Cassette {
    pub chr_rom: Vec<u8>,                                // CHR ROM ... 8KB or 16KB
    // pub chr_ram: Vec<u8>,                                // CHR-RAM (Ext RAM)
    pub prg_rom: Vec<u8>,                                // PRG ROM ... 8KB ~ 1MB
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Cassette {
    pub fn new() -> Self {
        Cassette {
            chr_rom: vec![0; CHR_ROM_MIN_SIZE],
            // chr_ram: vec![0; CHR_RAM_MIN_SIZE],
            prg_rom: vec![0; PRG_ROM_MIN_SIZE],
            mapper: 0,
            screen_mirroring: Mirroring::VERTICAL,
        }
    }

    pub fn rom_load(raw: &Vec<u8>) -> Result<Cassette, String> {
        let prg_rom_size = raw[4] as usize * PRG_ROM_MIN_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_MIN_SIZE;

        // iNES Check
        if &raw[0..4] != NES_ASCII {
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
                (_, _) => panic!("[ERR]: But Cassette iNES !!!"),
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

        Ok(Cassette {
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            // TODO :CHR-RAM
            // chr_ram: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        })
    }
}

pub fn rom_loader(cassette :&mut Cassette, path :&str)
{
    let mut f = File::open(path).expect("[ERR]: No File Found!!!");
    let metadata = std::fs::metadata(path).expect("[ERR]: Unable Read Metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("[ERR]: File Read Buffer OVF");

    *cassette = Cassette::rom_load(&buffer).expect("[ERR]: ROM Read ERROR");

    // (DEBUG)
    rom_info_dump(&path, cassette)
}

// (DEBUG) ROM info Dump
fn rom_info_dump(path:&str, cassette :&Cassette)
{
    let filename = Path::new(path)
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or("");

    println!("*********** [NES ROM Info] ***********");
    println!("NES File: {}", filename);
    println!("PRG-ROM Size: {} bytes", cassette.prg_rom.len());
    println!("CHR-ROM Size: {} bytes", cassette.chr_rom.len());
    println!("Mapper: {}", cassette.mapper);
    println!("Screen Mirroring: {:X?}", cassette.screen_mirroring);
    println!("****************************************\n");
}
// ====================================== TEST ======================================
mod rom_test {
    use super::*;

    #[test]
    fn rom_test() {
        for path in ROM_MAPPER_0_ROM_TBL.iter() {
            let mut cassette = Cassette::new();
            rom_loader(&mut cassette, path);
        }
    }
}
// ==================================================================================