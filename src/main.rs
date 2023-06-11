mod mem;
mod cpu;
mod ppu;
mod apu;
mod cassette;
use crate::cpu::RP2A03;
use cpu::{cpu_reset, cpu_main};
use ppu::{ppu_reset, ppu_main};
use apu::{apu_reset, apu_main};
use std::thread;
use std::time::Duration;
use rand::Rng;
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}
fn read_screen_state(cpu_rp2a03: &mut RP2A03, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    let mut rng = rand::thread_rng();
    cpu_rp2a03.nes_mem.mem_write(0xfe, rng.gen_range(1..16) as u8);

    for i in 0x0200..0x600 {
        let color_idx = cpu_rp2a03.nes_mem.mem_read(i as u16);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }
    update
}

fn handle_user_input(cpu_rp2a03: &mut RP2A03, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                std::process::exit(0)
            },
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                cpu_rp2a03.nes_mem.mem_write(0xff, 0x77);
            },
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                cpu_rp2a03.nes_mem.mem_write(0xff, 0x73);
            },
            Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                cpu_rp2a03.nes_mem.mem_write(0xff, 0x61);
            },
            Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                cpu_rp2a03.nes_mem.mem_write(0xff, 0x64);
            },
            _ => {/* do nothing */}
        }
    }
}


fn app_init()
{
    // TODO : App Init
}

fn main()
{
    // init sdl2
    let mut screen_state = [0 as u8; 32 * 3 * 32];
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
        .position_centered()
        .build().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(10.0, 10.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 32, 32).unwrap();
// ==================================================================================
    // [H/W Reset & App Init]
    let mut cpu_handler = cpu_reset();
    ppu_reset();
    apu_reset();
    app_init();
    // ==================================================================================
    // [Thred Main Loop]

    // CPU Thred @1.79 MHz(558.6 nsec)
    let _cpu_thread = thread::spawn(|| {
        loop {
            cpu_main();
            thread::sleep(Duration::from_nanos(559));
        }
    });

    // PPU Thred @5.37 MHz(186.4 nsec)
    // let _ppu_thread = thread::spawn(|| {
    //     loop {
    //         ppu_main();
    //         thread::sleep(Duration::from_nanos(187));
    //     }
    // });

    // let _apu_thread = thread::spawn(|| {
    //     loop {
    //         apu_main();
    //         thread::sleep(Duration::from_millis(500));
    //     }
    // });

// ==================================================================================
// [Main Loop]
    loop {
        handle_user_input(&mut cpu_handler, &mut event_pump);

        if read_screen_state(&mut cpu_handler, &mut screen_state) {
            texture.update(None, &screen_state, 32 * 3).unwrap();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        }
        thread::sleep(Duration::from_nanos(559));
    }
// ==================================================================================
}