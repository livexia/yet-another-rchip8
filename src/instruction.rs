use std::fmt;

pub struct Instruction {
    pub opcode: u16,
}

impl Instruction {
    pub fn new(high: u8, low: u8) -> Self {
        Instruction {
            opcode: (high as u16) << 8 | low as u16,
        }
    }

    pub fn kind(&self) -> u8 {
        (self.opcode >> 12 & 0x0f) as u8
    }

    pub fn x(&self) -> usize {
        (self.opcode >> 8 & 0x0f) as usize
    }

    pub fn y(&self) -> usize {
        (self.opcode >> 4 & 0x0f) as usize
    }

    pub fn n(&self) -> u8 {
        (self.opcode & 0x0f) as u8
    }

    pub fn nn(&self) -> u8 {
        (self.opcode & 0xff) as u8
    }

    pub fn nnn(&self) -> u16 {
        self.opcode & 0xfff
    }

    pub fn decode(&self) -> (u8, usize, usize, u8, u8, u16) {
        (
            self.kind(),
            self.x(),
            self.y(),
            self.n(),
            self.nn(),
            self.nnn(),
        )
    }
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instruction")
            .field("opcode", &self.opcode)
            .field("kind", &self.kind())
            .field("x", &self.x())
            .field("y", &self.y())
            .field("n", &self.n())
            .field("nn", &self.nn())
            .field("nnn", &self.nnn())
            .finish()
    }
}
