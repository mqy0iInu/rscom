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

    // [OK]
    // [MapperMMC 0]
    // let rom = load_rom("rom/nes/mapper_0/Alter_Ego.nes");
    // let rom = load_rom("rom/nes/mapper_0/BombSweeper.nes");
    // let rom = load_rom("rom/nes/mapper_0/pacman.nes");
    // let rom = load_rom("rom/nes/mapper_0/Super_Mario_Bros.nes");
    // let rom = load_rom("rom/nes/mapper_0/popeye.nes");
    // let rom = load_rom("rom/nes/mapper_0/excitebike.nes");

    // [MapperMMC 1]
    // let rom = load_rom("rom/nes/Dragon Quest 3 (J).nes");
    // let rom = load_rom("rom/nes/Dragon Quest 4 (J).nes");

    // [MapperMMC 2]
    // let rom = load_rom("rom/nes/Dragon Quest 2 (J).nes");

    // [MapperMMC 3]
    let rom = load_rom("rom/nes/Dragon Quest.nes");


    // [NG]
    // [MapperMMC 0]
    // let rom = load_rom("rom/nes/mapper_0/donkeykong.nes");
    // let rom = load_rom("rom/nes/mapper_0/xevious.nes");
    // let rom = load_rom("rom/nes/mapper_0/golf.nes");
    // let rom = load_rom("rom/nes/mapper_0/tower_of_druaga.nes");
    // let rom = load_rom("rom/nes/mapper_0/mario_bros.nes");

    MAPPER.lock().unwrap().prg_rom = rom.prg_rom.clone();
    MAPPER.lock().unwrap().chr_rom = rom.chr_rom.clone();
    MAPPER.lock().unwrap().mapper = rom.mapper.clone();

    info!(
        "ROM: mapper={}, mirroring={:?} chr_ram={}",
        rom.mapper, rom.screen_mirroring, rom.is_chr_ram
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