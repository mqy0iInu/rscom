use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Button: u8 {
        const RIGHT      = 0b1000_0000;
        const LEFT       = 0b0100_0000;
        const DOWN       = 0b0010_0000;
        const UP         = 0b0001_0000;
        const START      = 0b0000_1000;
        const SELECT     = 0b0000_0100;
        const BUTTON_B   = 0b0000_0010;
        const BUTTON_A   = 0b0000_0001;
    }
}

pub struct GamePad {
    strobe: bool,
    btn_index: u8,
    btn: Button,
    key_map: HashMap<Keycode, Button>,
}

impl GamePad {
    pub fn new() -> Self {
        GamePad {
            strobe: false,
            btn_index: 0,
            btn: Button::from_bits_truncate(0),
            key_map: HashMap::new(),
        }
    }

    fn gamepad_init(&mut self)
    {
        self.key_map.insert(Keycode::Down, Button::DOWN);
        self.key_map.insert(Keycode::Up, Button::UP);
        self.key_map.insert(Keycode::Right, Button::RIGHT);
        self.key_map.insert(Keycode::Left, Button::LEFT);
        self.key_map.insert(Keycode::Space, Button::SELECT);
        self.key_map.insert(Keycode::Return, Button::START);
        self.key_map.insert(Keycode::A, Button::BUTTON_A);
        self.key_map.insert(Keycode::S, Button::BUTTON_B);
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data & 1 == 1;
        if self.strobe {
            self.btn_index = 0
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.btn_index > 7 {
            return 1;
        }

        let response = (self.btn.bits() & (1 << self.btn_index)) >> self.btn_index;
        if !self.strobe && self.btn_index <= 7 {
                self.btn_index += 1;
        }
        response
    }

    pub fn set_button_pressed_status(&mut self, button: Button, value: bool) {
        self.btn.set(button, value)
    }
}