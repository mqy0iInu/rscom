mod mem;
mod cpu;
mod ppu;
mod apu;
mod cassette;
use cpu::{cpu_reset, cpu_main};
use ppu::{ppu_reset, ppu_main};
use apu::{apu_reset, apu_main};
use std::thread;
use std::time::Duration;

fn app_init()
{
    // TODO : App Init
    cpu_reset();
    ppu_reset();
    apu_reset();
}

fn main()
{
// ==================================================================================
    // [H/W Reset & App Init]
    app_init();

// ==================================================================================
// [Thred Main Loop]
    let _cpu_thread = thread::spawn(|| {
        loop {
            cpu_main();
            thread::sleep(Duration::from_millis(300));
        }
    });

    let _ppu_thread = thread::spawn(|| {
        loop {
            ppu_main();
            thread::sleep(Duration::from_millis(300));
        }
    });

    let _apu_thread = thread::spawn(|| {
        loop {
            apu_main();
            thread::sleep(Duration::from_millis(300));
        }
    });

// ==================================================================================
// [Main Loop]
    loop {
        // println!("[DEBUG] : App Main Loop");
        thread::sleep(Duration::from_millis(300));
    }
// ==================================================================================
}