use std::error::Error;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{EventPump, Sdl};

use crossbeam_channel::{select, unbounded};
use rand::Rng;

use crate::audio::Audio;
use crate::font::DEFAULTFONT;
use crate::instruction::Instruction;
use crate::keyboard::{KeyBoard, KeyMap};
use crate::rom::ROM;
use crate::video::Video;
use crate::{err, Result};

const MEMORY_SIZE: usize = 4096;
const RESERVED_MEMORY_SIZE: usize = 512;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

pub struct Machine {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT],
    pc: u16,
    // index register
    i: u16,
    stack: Vec<u16>, // TODO: should be [u16; 16] and with a stack pointer
    delay_timer: u8,
    sound_timer: u8,
    keyboard: KeyBoard,
    sdl_context: Sdl,
    video: Video,
    audio: Audio,
}

impl Machine {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = Video::new(sdl_context.video()?, 64, 32)?;
        let audio_subsystem = Audio::new(sdl_context.audio()?)?;

        Ok(Machine {
            memory: [0; MEMORY_SIZE],
            registers: [0; REGISTER_COUNT],
            pc: 0x200,
            i: 0x0,
            stack: Vec::with_capacity(STACK_SIZE),
            delay_timer: 0,
            sound_timer: 0,
            sdl_context,
            keyboard: KeyBoard::default(),
            video: video_subsystem,
            audio: audio_subsystem,
        })
    }

    pub fn load_font(&mut self) -> Result<()> {
        // TODO: load from file
        self.memory[0x50..0x50 + DEFAULTFONT.len()].copy_from_slice(&DEFAULTFONT[..]);
        Ok(())
    }

    pub fn load_rom(&mut self, rom: &ROM) -> Result<()> {
        if rom.len() > MEMORY_SIZE - RESERVED_MEMORY_SIZE {
            return err!(
                "can not load rom({} Bytes) that big than the machine memory({} Bytes)",
                rom.len(),
                self.memory.len()
            );
        }
        let start = self.pc as usize;
        let end = start + rom.len();
        self.memory[start..end].clone_from_slice(&rom.raw()[..]);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let (timer_tx, timer_rx) = unbounded();
        let (clock_tx, clock_rx) = unbounded();

        // timer 60Hz ~= 16667 micros
        let timer_dur = Duration::from_micros(1000000 / 60);
        thread::spawn(move || loop {
            thread::sleep(timer_dur);
            timer_tx.send(chrono::Utc::now()).unwrap();
        });
        // clock 500Hz ~= 2000 micros
        let clock_dur = Duration::from_micros(1000000 / 500);
        thread::spawn(move || loop {
            thread::sleep(clock_dur);
            clock_tx.send(chrono::Utc::now()).unwrap();
        });

        let mut running = true;
        let mut event_pump = self.sdl_context.event_pump()?;
        let key_map = KeyMap::default();
        while running && (self.pc as usize) < MEMORY_SIZE - 1 {
            select! {
                recv(timer_rx) -> msg => {
                    if self.delay_timer > 0 {
                        self.delay_timer -= 1;
                    };
                    if self.sound_timer > 0 {
                        self.audio.resume();
                        self.sound_timer -= 1;
                    } else {
                        self.audio.pause();
                    };
                    // TODO: draw sdl2 canvas based of CHIP.video
                    self.video.draw()?;
                    debug!("timer: {}", msg.unwrap());
                },
                recv(clock_rx) -> msg => {
                    self.process_key_event(&mut running, &mut event_pump, &key_map);
                    self.run_cycle()?;
                    debug!("clock: {}", msg.unwrap());
                    debug!("registers: {:02?}", self.registers);
                },
            };
        }
        Ok(())
    }

    fn process_key_event(
        &mut self,
        running: &mut bool,
        event_pump: &mut EventPump,
        key_map: &KeyMap,
    ) {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    *running = false;
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(key) = key_map.scancode_to_key(&scancode) {
                        self.keyboard.key_down(key);
                        debug!("KeyDown: {:?} -> {}", scancode, key);
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    if let Some(key) = key_map.scancode_to_key(&scancode) {
                        self.keyboard.key_up(key);
                        debug!("KeyUp: {:?} -> {}", scancode, key);
                    }
                }
                _ => {}
            }
        }
    }

    fn fetch(&mut self) -> Result<Instruction> {
        let instr = Instruction::new(
            self.memory[self.pc as usize],
            self.memory[self.pc as usize + 1],
        );
        self.pc += 2;
        Ok(instr)
    }

    fn run_cycle(&mut self) -> Result<()> {
        let instr = self.fetch()?;
        debug!("execute: {:04X}, pc: {:04X}", instr.opcode, self.pc - 2);
        let opcode = instr.opcode;
        let (kind, x, y, n, nn, nnn) = instr.decode();
        match kind {
            0x0 => {
                if opcode == 0x00e0 {
                    self.video.clear();
                } else if opcode == 0x00ee {
                    self.pc = self.stack.pop().unwrap(); // TODO: 需要后续编写错误处理
                }
            }
            0x1 => self.pc = nnn,
            0x2 => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            0x3 => {
                if self.registers[x] == nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.registers[x] != nn {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            0x6 => {
                self.registers[x] = nn;
            }
            0x7 => {
                self.registers[x] = self.registers[x].overflowing_add(nn).0;
            }
            0x8 => {
                //8XYN
                match n {
                    0x0 => self.registers[x] = self.registers[y],
                    0x1 => self.registers[x] |= self.registers[y],
                    0x2 => self.registers[x] &= self.registers[y],
                    0x3 => self.registers[x] ^= self.registers[y],
                    0x4 => self.add(x, y),  // 8xy4
                    0x5 => self.sub(x, y),  // 8xy5
                    0x7 => self.subb(x, y), // 8xy7
                    0x6 => {
                        //ignore the y
                        self.registers[0xf] = self.registers[x] & 1;
                        self.registers[x] >>= 1;
                    }
                    0xe => {
                        //ignore the y
                        self.registers[0xf] = self.registers[x] >> 7;
                        self.registers[x] <<= 1;
                    }
                    _ => (),
                }
            }
            0x9 => {
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }
            0xA => {
                self.i = nnn;
            }
            0xB => {
                self.pc = nnn + self.registers[0] as u16;
            }
            0xC => {
                let mut rng = rand::thread_rng();
                let r1: u8 = rng.gen();
                self.registers[x] = r1 & nn;
            }
            0xD => {
                let x = (self.registers[x] % 64) as usize;
                let y = (self.registers[y] % 32) as usize;
                debug!("draw at: ({}, {})", x, y);
                let n = n as usize;
                self.registers[0xf] =
                    self.video
                        .flip(x, y, n, &self.memory[self.i as usize..self.i as usize + n])
            }
            0xE => {
                let key = self.registers[x];
                let required_key_pressed = self.keyboard.is_key_down(key);
                match (required_key_pressed, nn) {
                    (true, 0x9E) => {
                        self.pc += 2;
                        info!("instr: {:04X}, key {:X?} pressed", opcode, key)
                    }
                    (false, 0xA1) => {
                        self.pc += 2;
                        info!("instr: {:04X}, key {:X?} not pressed", opcode, key)
                    }
                    _ => (),
                }
            }
            0xF => match nn {
                0x7 => self.registers[x] = self.delay_timer,
                0x15 => self.delay_timer = self.registers[x],
                0x18 => self.sound_timer = self.registers[x],
                0x1E => self.i += self.registers[x] as u16,
                0x0A => {
                    if let Some(pressed_key) = self.keyboard.first_down_key() {
                        self.registers[x] = pressed_key;
                        info!("key {:X} is being pressed", pressed_key);
                        // after pressed, key should be up. https://github.com/livexia/yet-another-rchip8/issues/10#issue-1713963954
                        self.keyboard.key_up(pressed_key);
                    } else {
                        self.pc -= 2;
                    }
                }
                0x29 => {
                    let char = self.registers[x];
                    self.i = 0x50 + 5 * char as u16;
                    debug!("look char: {:X}", char);
                }
                0x33 => {
                    let mut x_val = self.registers[x];
                    self.memory[self.i as usize + 2] = x_val % 10;
                    x_val /= 10;
                    self.memory[self.i as usize + 1] = x_val % 10;
                    x_val /= 10;
                    self.memory[self.i as usize] = x_val;
                    debug!(
                        "x: {}, BCD: {:?}",
                        self.registers[x],
                        &self.memory[self.i as usize..self.i as usize + 3]
                    );
                }
                0x55 => {
                    let i = self.i as usize;
                    self.memory[i..=i + x].copy_from_slice(&self.registers[..=x]);
                }
                0x65 => {
                    let i = self.i as usize;
                    self.registers[..=x].copy_from_slice(&self.memory[i..=i + x]);
                }
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    /// 8xy4
    fn add(&mut self, x: usize, y: usize) {
        let (val, flag) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[0xf] = flag as u8;
        self.registers[x] = val;
    }

    /// 8xy5
    fn sub(&mut self, x: usize, y: usize) {
        let (val, flag) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[0xf] = (!flag) as u8;
        self.registers[x] = val;
    }

    /// 8xy7
    fn subb(&mut self, x: usize, y: usize) {
        let (val, flag) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[0xf] = (!flag) as u8;
        self.registers[x] = val;
    }
}
