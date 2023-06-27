use crate::{common};
use common::*;

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A]; // NES^Z
const PRG_ROM_PAGE_SIZE: usize = 16 * 1024; // 16KiB
const CHR_ROM_PAGE_SIZE: usize = 8 * 1024; // 8KiB

#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mirroring {
    VERTICAL,
    HORIZONTAL,
    FOUR_SCREEN,
    ONE_SCREEN_LOWER,
    ONE_SCREEN_UPPER,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(non_camel_case_types)]
pub enum RomType {
    NROM,  // MMC0 Mapper 0(Mario)
    BXROM, // (TBD)
    UXROM, // Mapper 2(DQ2)/94/180
    SNROM, // MMC1 Mapper 1 (DQ3, Zelda)
    SXROM, // MMC1 Mapper 1
    SOROM, // MMC1 Mapper 1
    SUROM, // MMC1 Mapper 1 (DQ4)
    CNROM, // Mapper 3 (DQ)
    TKROM, // MMC3 Mapper 4 (Mother)/118
    TLROM, // MMC3 Mapper 4/118
    TXROM, // MMC3 Mapper 4
    TQROM, // MMC3 Mapper 119
    UNKNOWN
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub is_chr_ram: bool,
    pub is_prg_ram: bool,
    pub rom_type: RomType,
}

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Rom, String> {
        Self:: mem_blank();

        if &raw[0..4] != NES_TAG {
            return Err("File is not in iNES file format".to_string());
        }

        let mapper = (raw[7] & 0xF0) | (raw[6] >> 4);

        // let ines_ver = (raw[7] >> 2) & 0b11;
        // if ines_ver != 0 {
        //     return Err("NES2.0 format is not supported".to_string());
        // }

        // MMMMftcm
        // ||||||||
        // |||||||+- Mirroring: 0: horizontal (vertical arrangement) (CIRAM A10 = PPU A11)
        // |||||||              1: vertical (horizontal arrangement) (CIRAM A10 = PPU A10)
        // ||||||+-- 1: カートリッジにバッテリバックアップされたPRG RAM ($6000-7FFF)またはその他の永続メモリが搭載
        // |||||+--- 1: $7000-$71FFにある512バイトのトレーナー（PRGデータの前に格納される）
        // ||||+---- 1：ミラーリング制御または上記ミラーリングビットを無視し、代わりに4画面VRAMを提供する
        // ++++----- マッパー番号の下位ニブル
        let four_screen = raw[6] & _BIT_3 != 0;
        let is_prg_ram: bool = raw[6] & _BIT_1 != 0;
        let mirroring_type = raw[6] & _BIT_0 != 0;
        let mirroring = match (four_screen, mirroring_type) {
            (true, _) => Mirroring::FOUR_SCREEN,
            (false, true) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let chr_rom = if chr_rom_size == 0 {
            // chr_rom_size=0の場合、8KBのCHR_RAMが存在する
            let blank_chr_ram: Vec<u8> = vec![0; CHR_ROM_PAGE_SIZE];
            blank_chr_ram
        } else {
            raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec()
        };

        // TODO :ROMタイプ判別
        let mut rom_type: RomType = RomType::UNKNOWN;
        match mapper {
            0 => rom_type = RomType::NROM,
            1 => { if (prg_rom_size >= (_MEM_SIZE_512K as usize))
                && (chr_rom_size == 0) && (is_prg_ram != false) {
                    rom_type = RomType::SUROM;
                }else{
                    rom_type = RomType::SNROM;
                }
            },
            2 => { if (prg_rom_size <= (_MEM_SIZE_128K as usize))
                && (chr_rom_size == 0) && (is_prg_ram != true) {
                    rom_type = RomType::UXROM;
                }
            },
            3 => { if (chr_rom_size != 0) && (is_prg_ram != true) {
                    rom_type = RomType::CNROM;
                }
            },
            4 => { if (prg_rom_size >= (_MEM_SIZE_256K as usize))
                && (chr_rom_size >= (_MEM_SIZE_128K as usize))
                && (is_prg_ram != false) {
                    rom_type = RomType::TKROM;
                }
            },
            _ => panic!("[ERR] Unknown ROM Type (Mapper: {}, ROM Type: {:?})",mapper, rom_type),
        }

        Ok(Rom {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: chr_rom,
            mapper: mapper,
            mirroring: mirroring,
            is_chr_ram: chr_rom_size == 0,
            is_prg_ram: is_prg_ram,
            rom_type: rom_type,
        })
    }

    pub fn mem_blank() -> Self {
        return Rom {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper: 0,
            mirroring: Mirroring::VERTICAL,
            is_chr_ram: false,
            is_prg_ram: false,
            rom_type: RomType::NROM,
        };
    }
}
