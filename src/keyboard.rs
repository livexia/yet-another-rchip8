pub struct KeyBoard {
    keys: [bool; 16],
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

impl Default for KeyBoard {
    fn default() -> Self {
        Self::new()
    }
}
