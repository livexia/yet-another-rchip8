use std::fs::File;
use std::io::Read;

use crate::Result;

#[derive(Debug)]
pub struct ROM {
    pub name: String,
    raw: Vec<u8>,
    length: usize,
}

impl ROM {
    pub fn new(path: &str) -> Result<Self> {
        let mut temp_f = File::open(path)?;
        let mut raw = Vec::new();
        temp_f.read_to_end(&mut raw)?;
        let length = raw.len();
        Ok(ROM {
            name: path.to_string(),
            raw,
            length,
        })
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn raw(&self) -> Vec<u8> {
        self.raw.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}
