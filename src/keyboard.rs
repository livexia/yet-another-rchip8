use std::collections::HashMap;
use std::error::Error;

use sdl2::keyboard::Scancode;

use crate::err;
use crate::Result;

pub struct KeyBoard {
    keys_map: HashMap<u8, Scancode>,
    scancodes_map: HashMap<Scancode, u8>,
    keys: [bool; 16],
}

impl KeyBoard {
    pub fn new(layout: &HashMap<u8, Scancode>) -> Result<Self> {
        let keys_map = layout.clone();
        let mut scancodes_map = HashMap::with_capacity(16);
        if layout.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        for (&key, &scancode) in layout {
            scancodes_map.insert(scancode, key);
        }
        if keys_map.len() != 16 || scancodes_map.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        Ok(KeyBoard {
            keys_map,
            scancodes_map,
            keys: [false; 16],
        })
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

    pub fn key_to_scancode(&self, key: &u8) -> Option<Scancode> {
        self.keys_map.get(key).copied()
    }

    pub fn scancode_to_key(&self, scancode: &Scancode) -> Option<u8> {
        self.scancodes_map.get(scancode).copied()
    }

    fn default_keyboard_layout() -> HashMap<u8, Scancode> {
        let mut default_layout: HashMap<u8, Scancode> = HashMap::new();
        default_layout.insert(0, Scancode::X);
        default_layout.insert(1, Scancode::Num1);
        default_layout.insert(2, Scancode::Num2);
        default_layout.insert(3, Scancode::Num3);
        default_layout.insert(4, Scancode::Q);
        default_layout.insert(5, Scancode::W);
        default_layout.insert(6, Scancode::E);
        default_layout.insert(7, Scancode::A);
        default_layout.insert(8, Scancode::S);
        default_layout.insert(9, Scancode::D);
        default_layout.insert(0xA, Scancode::Z);
        default_layout.insert(0xB, Scancode::C);
        default_layout.insert(0xC, Scancode::Num4);
        default_layout.insert(0xD, Scancode::R);
        default_layout.insert(0xE, Scancode::F);
        default_layout.insert(0xF, Scancode::V);
        default_layout
    }
}

impl Default for KeyBoard {
    fn default() -> Self {
        Self::new(&Self::default_keyboard_layout()).unwrap()
    }
}
