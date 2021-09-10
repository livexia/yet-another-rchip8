use std::result;
use std::error::Error;

use std::collections::HashMap;
use std::collections::HashSet;

use sdl2::keyboard::Scancode;

macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) };
}

type Result<T> = result::Result<T, Box<dyn Error>>;
pub struct KeyBoard {
    keys_map: HashMap<u8, Scancode>,
    scancodes_map: HashMap<Scancode, u8>
}

impl KeyBoard {
    pub fn new(layout: &HashMap<u8, Scancode>) -> Result<Self>{
        let mut keys_map = HashMap::with_capacity(16);
        let mut scancodes_map = HashMap::with_capacity(16);
        if layout.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        keys_map = layout.clone();
        for (&key, &scancode) in layout {
            scancodes_map.insert(scancode, key);
        }
        if keys_map.len() != 16 || scancodes_map.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        Ok(KeyBoard{
            keys_map, scancodes_map
        })
    }

    pub fn default() -> Self {
        Self::new(&Self::default_keyboard_layout()).unwrap()
    }

    pub fn key_to_scancode(&self, key: &u8) -> Option<Scancode> {
        match self.keys_map.get(key) {
            Some(&k) => Some(k),
            None => None,
        }
    }

    pub fn scancode_to_key(&self, scancode: &Scancode) -> Option<u8> {
        match self.scancodes_map.get(scancode) {
            Some(&k) => Some(k),
            None => None,
        }
    }
    
    fn default_keyboard_layout() -> HashMap<u8, Scancode>{
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