mod mem;
mod cpu;
mod ppu;
mod apu;
mod cassette;
mod gamepad;
use cpu::{cpu_reset, cpu_main};
use ppu::{ppu_reset, ppu_main};
use apu::{apu_reset, apu_main};
use std::thread;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;


fn app_init()
{
    // TODO : App Init(TBD ... エミュレータGUI関連 etc.)
}

fn main()
{
    // ==================================================================================
    // [H/W Reset & App Init]
    let sdl_context = sdl2::init().unwrap();

    let mut _cpu_handler = cpu_reset();
    let mut _ppu_handler = ppu_reset(sdl_context);
    apu_reset();

    app_init();
    // ==================================================================================
    // [Thred Main Loop]

    // 元の周波数 = 21.47727 MHz
    // クロック分周比 = 12
    // CPUの周波数 = 1.7897725 MHz
    // T = 558.7302296800292 nsec
    // CPU Thred @1.79 MHz(558.55 nsec)
    let _cpu_thread = thread::spawn(|| {
        loop {
            cpu_main();
            // thread::sleep(Duration::from_nanos(559));
            thread::sleep(Duration::from_micros(300));
            // thread::sleep(Duration::from_millis(6));
        }
    });

    // 元の周波数 = 21.47727 MHz
    // クロック分周比: 4
    // PPUの周波数 = 5.3693175 MHz
    // T = 186.2434098933431 nsec
    // PPU Thred @5.37 MHz(186.41 nsec)
    let _ppu_thread = thread::spawn(|| {
        loop {
            ppu_main();
            // thread::sleep(Duration::from_nanos(187));
            thread::sleep(Duration::from_millis(100));
        }
    });

    // let _apu_thread = thread::spawn(|| {
    //     loop {
    //         apu_main();
    //         // thread::sleep(Duration::from_nanos(559));
    //         thread::sleep(Duration::from_millis(500));
    //     }
    // });

    // ==================================================================================
    // [Main Loop]

    loop {
        // TODO :メインループ(TBD ... エミュレータGUI関連 etc.)

        // thread::sleep(Duration::from_nanos(187));
        thread::sleep(Duration::from_secs(5));
        // thread::sleep(Duration::from_millis(2));
    }
    // ==================================================================================
}