use std::collections::HashMap;
use std::error::Error;

use sdl2::keyboard::Scancode;

use crate::err;
use crate::Result;

pub struct KeyBoard {
    keys: [bool; 16],
}

pub struct KeyMap {
    scancodes_map: HashMap<Scancode, u8>,
}

impl KeyBoard {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn key_down(&mut self, key: u8) {
        self.keys[key as usize] = true;
    }

    pub fn key_up(&mut self, key: u8) {
        self.keys[key as usize] = false;
    }

    pub fn is_key_down(&self, key: u8) -> bool {
        self.keys[key as usize]
    }

    pub fn first_down_key(&self) -> Option<u8> {
        self.keys
            .iter()
            .copied()
            .enumerate()
            .find(|(_, b)| *b)
            .map(|(i, _)| i as u8)
    }
}

impl KeyMap {
    pub fn new(layout: &HashMap<Scancode, u8>) -> Result<Self> {
        let scancodes_map = layout.clone();
        if layout.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        Ok(KeyMap { scancodes_map })
    }

    pub fn scancode_to_key(&self, scancode: &Scancode) -> Option<u8> {
        self.scancodes_map.get(scancode).copied()
    }

    fn default_keyboard_layout() -> HashMap<Scancode, u8> {
        let mut default_layout: HashMap<Scancode, u8> = HashMap::with_capacity(16);
        default_layout.insert(Scancode::X, 0);
        default_layout.insert(Scancode::Num1, 1);
        default_layout.insert(Scancode::Num2, 2);
        default_layout.insert(Scancode::Num3, 3);
        default_layout.insert(Scancode::Q, 4);
        default_layout.insert(Scancode::W, 5);
        default_layout.insert(Scancode::E, 6);
        default_layout.insert(Scancode::A, 7);
        default_layout.insert(Scancode::S, 8);
        default_layout.insert(Scancode::D, 9);
        default_layout.insert(Scancode::Z, 0xA);
        default_layout.insert(Scancode::C, 0xB);
        default_layout.insert(Scancode::Num4, 0xC);
        default_layout.insert(Scancode::R, 0xD);
        default_layout.insert(Scancode::F, 0xE);
        default_layout.insert(Scancode::V, 0xF);
        default_layout
    }
}

impl Default for KeyBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::new(&Self::default_keyboard_layout()).unwrap()
    }
}
