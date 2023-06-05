mod cpu;
mod ppu;
mod apu;
use cpu::{cpu_main, cpu_reset};
use ppu::{ppu_main, ppu_reset};
use apu::{apu_main, apu_reset};
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
            thread::sleep(Duration::from_millis(500));
        }
    });

    let _apu_thread = thread::spawn(|| {
        loop {
            apu_main();
            thread::sleep(Duration::from_millis(800));
        }
    });

// ==================================================================================
// [Main Loop]
    loop {
        println!("[DEBUG] : App Main Loop");
        thread::sleep(Duration::from_millis(999));
    }
// ==================================================================================
}