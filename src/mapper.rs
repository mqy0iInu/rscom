const MAPPER_2_PRG_ROM_BANK_SIZE: usize = 16 * 1024;
const MAPPER_3_CHR_ROM_BANK_SIZE: usize = 8 * 1024;

pub struct MapperMMC {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    bank_select: u8,
    pub mapper: u8,
}

impl MapperMMC {
    pub fn new() -> Self {
        MapperMMC {
            prg_rom: vec![],
            chr_rom: vec![],
            bank_select: 0,
            mapper: 2,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bank_select = data;
    }

    fn mapper_2_read(&self, addr: u16) -> u8 {
        let bank_len = MAPPER_2_PRG_ROM_BANK_SIZE;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x8000..=0xBFFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.prg_rom[(addr as usize - 0x8000 + bank_len * bank as usize) as usize]
            },
            0xC000..=0xFFFF => {
                // 最後のバンク固定
                self.prg_rom[(addr as usize - 0xC000 + bank_len * (bank_max - 1)) as usize]
            },
            _ => panic!("can't be"),
        }
    }

    fn mapper_3_prg_rom_read(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0xFFFF => {
                // 最後のバンク固定
                self.prg_rom[(addr - 0x8000)as usize]
            },
            _ => panic!("can't be"),
        }
    }


    fn mapper_3_chr_rom_read(&self, addr: u16) -> u8 {
        let bank_len = MAPPER_3_CHR_ROM_BANK_SIZE;
        let bank_max = self.prg_rom.len() / bank_len;
        match addr {
            0x0000..=0x1FFF => {
                // bank_select
                let bank = self.bank_select & 0x0F;
                self.chr_rom[(addr as usize + bank_len * bank as usize) as usize]
            },
            _ => panic!("can't be"),
        }
    }

    pub fn read_prg_rom(&mut self, mapper: u8, addr: u16) -> u8 {
        self.mapper = mapper;
        match mapper {
            0 | 2 => self.mapper_2_read(addr),
            3 => self.mapper_3_prg_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", mapper),
        }
    }

    pub fn read_chr_rom(&mut self, mapper: u8, addr: u16) -> u8 {
        self.mapper = mapper;
        match mapper {
            3 => self.mapper_3_chr_rom_read(addr),
            _ => panic!("[ERR] Not Emu Support MapperMMC {}", mapper),
        }
    }
}
