#[macro_use]
extern crate lazy_static;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod frame;
mod gamepad;
mod mapper;
mod opcode;
mod palette;
mod ppu;
mod render;
mod rom;
mod common;
use common::*;
use crate::cpu::{trace, IN_TRACE};

use self::bus::{Bus, Mem};
use self::cpu::CPU;

use apu::APU;
use cartridge::load_rom;
use frame::Frame;
use gamepad::GamePad;
use log::{debug, info, log_enabled, trace, Level};
use mapper::MapperMMC;
use ppu::PPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

lazy_static! {
    pub static ref MAPPER: Mutex<Box<MapperMMC>> = Mutex::new(Box::new(MapperMMC::new()));
}

fn main() {
    env_logger::builder()
        .format(|buf, record| {
            let style = buf.style();
            if unsafe { IN_TRACE } {
                writeln!(buf, "[TRACE] {}", style.value(record.args()))
            } else {
                writeln!(buf, "        {}", style.value(record.args()))
            }
        })
        .format_timestamp(None)
        .init();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rust NES Emulator", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(2.0, 2.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, gamepad::Button::DOWN);
    key_map.insert(Keycode::Up, gamepad::Button::UP);
    key_map.insert(Keycode::Right, gamepad::Button::RIGHT);
    key_map.insert(Keycode::Left, gamepad::Button::LEFT);
    key_map.insert(Keycode::Space, gamepad::Button::SELECT);
    key_map.insert(Keycode::Return, gamepad::Button::START);
    key_map.insert(Keycode::A, gamepad::Button::BUTTON_A);
    key_map.insert(Keycode::S, gamepad::Button::BUTTON_B);

    let rom = load_rom(_NES_ROM_PATH);
    MAPPER.lock().unwrap().prg_rom = rom.prg_rom.clone();
    MAPPER.lock().unwrap().chr_rom = rom.chr_rom.clone();
    MAPPER.lock().unwrap().is_chr_ram = rom.is_chr_ram.clone();
    MAPPER.lock().unwrap().is_prg_ram = rom.is_prg_ram.clone();
    MAPPER.lock().unwrap().mapper = rom.mapper.clone();
    MAPPER.lock().unwrap().rom_type = rom.rom_type.clone();
    MAPPER.lock().unwrap().mmc_1.rom_type = rom.rom_type.clone();

    info!(
        "ROM: mapper={}, mirroring={:?} chr_ram={}",
        rom.mapper, rom.mirroring, rom.is_chr_ram
    );

    let mut frame = Frame::new();
    let apu = APU::new(&sdl_context);
    let bus = Bus::new(rom, apu, move |ppu: &PPU, gamepad_1: &mut GamePad| {
        render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),
                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        gamepad_1.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        gamepad_1.set_button_pressed_status(*key, false);
                    }
                }
                _ => { /* do nothing */ }
            }
        }
    });

    let mut cpu = CPU::new(bus);

    cpu.reset();
    cpu.run_with_callback(move |cpu| {
        if log_enabled!(Level::Trace) {
            trace(cpu);
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ptr_from_vec() {
        let my_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        let ptr: *const u8 = my_vec.as_ptr();

        unsafe {
            let value: u8 = *ptr;
            assert_eq!(value, 1);
        }
    }

    #[test]
    fn test_ptr_print() {
        let my_vec: Vec<u8> = vec![1, 2, 3, 4, 5];
        let ptr: *const u8 = my_vec.as_ptr();

        unsafe {
            for i in 0..my_vec.len() {
                let value: u8 = *ptr.offset(i as isize);
                println!("Value at index {}: {}", i, value);
            }
        }
    }
}
