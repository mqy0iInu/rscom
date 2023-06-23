use crate::gamepad::GamePad;
use crate::ppu::PPU;
use crate::rom::Rom;
use crate::{apu::APU, MAPPER};
use log::{debug, error, info, log_enabled, trace, warn, Level};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    // prg_rom: Vec<u8>,
    ppu: PPU,
    gamepad_1: GamePad,
    gamepad_2: GamePad,
    apu: APU,

    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&PPU, &mut GamePad) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, apu: APU, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&PPU, &mut GamePad) + 'call,
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring, rom.is_chr_ram);
        Bus {
            cpu_vram: [0; 2048],
            // prg_rom: rom.prg_rom,
            ppu: ppu,
            gamepad_1: GamePad::new(),
            gamepad_2: GamePad::new(),
            apu: apu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    // fn read_prg_rom(&self, mut addr: u16) -> u8 {
    //     addr -= 0x8000;
    //     if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
    //         // mirror if needed
    //         addr = addr % 0x4000;
    //     }
    //     self.prg_rom[addr as usize]
    // }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        self.apu.tick(cycles);

        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&self.ppu, &mut self.gamepad_1);
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<i32> {
        if self.ppu.clear_nmi_interrupt {
            self.ppu.clear_nmi_interrupt = false;
            self.ppu.nmi_interrupt = None;
            return None;
        }
        let res = self.ppu.nmi_interrupt;
        self.ppu.nmi_interrupt = None;
        res
    }

    pub fn poll_apu_irq(&mut self) -> bool {
        self.apu.irq()
    }
}

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
}

impl Mem for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                let v = self.cpu_vram[mirror_down_addr as usize];
                trace!(
                    "RAM READ: {:04X} => {:04X} ({:02X})",
                    addr,
                    mirror_down_addr,
                    v
                );
                v
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                warn!("Attempt to read from write-only PPU address {:X}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                debug!("READ PPU MIRROR: {:04X} => {:04X}", addr, mirror_down_addr);
                self.mem_read(mirror_down_addr)
            }
            0x4015 => self.apu.read_status(),
            0x4016 => self.gamepad_1.read(),
            0x4017 => {
                // readはgamepad_2になるらしいが、writeはAPUらしい。。
                // self.gamepad_2.read()
                0
            }
            PRG_ROM..=PRG_ROM_END => {
                let mapper = MAPPER.lock().unwrap().mapper;
                MAPPER.lock().unwrap().read_prg_rom(mapper, addr)
            }
            _ => {
                warn!("Ignoreing mem access at {:X}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b_0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
                trace!(
                    "RAM WRITE: {:04X} => {:04X} ({:02X})",
                    addr,
                    mirror_down_addr,
                    data
                );
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => {
                self.ppu.write_to_status(data);
            }
            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }
            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }
            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            0x4000..=0x4003 => self.apu.write1ch(addr, data),
            0x4004..=0x4007 => self.apu.write2ch(addr, data),
            0x4008 | 0x400A | 0x400B => self.apu.write3ch(addr, data),
            0x400C | 0x400E | 0x400F => self.apu.write4ch(addr, data),
            0x4010..=0x4013 => {
                // TODO DMCch
            }
            0x4015 => {
                self.apu.write_status(data);
            }
            0x4016 => {
                self.gamepad_1.write(data);
            }
            0x4017 => {
                let pad_val = data;
                let apu_val = data & 0b1100_0000; // Bit[7:6]だけもらう
                self.gamepad_2.write(pad_val);
                self.apu.write_frame_counter(apu_val);
                info!("WRITE ACCESS 0x4017. {:02X}", data);
            }
            0x4014 => {
                // $XX を書き込むと、256 バイトのデータが
                // CPU ページ $XX00 ～ $XXFF から内部 PPU OAM にアップロードされます
                // このページは通常、内部 RAM (通常は $0200 ～ $02FF) にありますが、カートリッジ RAM または ROM も使用できます。
                let mut values: [u8; 256] = [0; 256];
                for i in 0x00..=0xFF {
                    values[i] = self.mem_read((data as u16) << 8 | i as u16);
                }
                self.ppu.write_to_oam_dma(values);
                // Not counting the OAMDMA write tick, the above procedure takes 513 CPU cycles (+1 on odd CPU cycles)
                for _ in 0..513 {
                    self.ppu.tick(1);
                }
            }
            PRG_ROM..=PRG_ROM_END => {
                MAPPER.lock().unwrap().write(addr, data);
                // warn!(
                //     "Attempt to write to Cartrige ROM space {:04X} => {:02X}",
                //     addr, data
                // )
            }
            _ => {
                error!("Ignoreing mem write-access at {:X}", addr)
            }
        }
    }
}
