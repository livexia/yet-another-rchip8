pub mod keyboard;
pub mod video;
pub mod audio;
pub mod rom;
pub mod instruction;

#[macro_use]
extern crate log;
extern crate sdl2;
extern crate clap;

use std::error::Error;
use std::ops::Shl;
use std::ops::Shr;
use std::result;
use std::collections::HashSet;
use std::time::Duration;
use std::thread;

use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use rand::Rng;
use chrono::Utc;
use clap::{Arg, App};
use crossbeam_channel::{select, unbounded};

use keyboard::KeyBoard;
use video::Video;
use audio::Audio;
use rom::ROM;
use instruction::Instruction;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) };
}

pub type Result<T> = result::Result<T, Box<dyn Error>>;

const MEMORY_SIZE: usize = 4096;
const RESERVED_MEMORY_SIZE: usize = 512;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

fn main() -> Result<()>{
    env_logger::init();

    let matches = App::new("yet-another-rchip8")
        .version("0.0001")
        .author("livexia")
        .arg(Arg::with_name("ROM")
            .short("r")
            .long("rom")
            .takes_value(true)
            .help("Sets the rom file to load"))
        .get_matches();
    
    let rom = matches.value_of("ROM").unwrap_or("IBM_Logo.hex");
    let rom = ROM::new(rom)?;
    let mut machine = Machine::new()?;
    machine.load_font()?;
    machine.load_rom(&rom)?;
    machine.run()?;
    Ok(())
}

struct Machine {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    keyboard: KeyBoard,
    sdl_context: Sdl,
    video: Video,
    audio: Audio
}

impl Machine {
    pub fn new() -> Result<Self>{
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
            audio: audio_subsystem
        })
    }

    pub fn load_font(&mut self) -> Result<()> {
        let font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80
        ];
        for i in 0..font.len() {
            self.memory[0x50 + i] = font[i];
        }
        Ok(())
    }

    pub fn load_rom(&mut self, rom: &ROM) -> Result<()> {
        if rom.len() > MEMORY_SIZE - RESERVED_MEMORY_SIZE {
            return err!("can not load rom({} Bytes) that big than the machine memory({} Bytes)", rom.len(), self.memory.len());
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
        let timer_dur = Duration::from_micros(16657);
        thread::spawn(move || {
            loop {
                thread::sleep(timer_dur);
                timer_tx.send(chrono::Utc::now()).unwrap();
            }
        });
        // clock 500Hz ~= 2000 micros
        let clock_dur = Duration::from_micros(2000);
        thread::spawn(move || {
            loop {
                thread::sleep(clock_dur);
                clock_tx.send(chrono::Utc::now()).unwrap();
            }
        });

        let mut running = true;
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
                    self.video.draw()?;
                    debug!("timer: {}", msg.unwrap());
                },
                recv(clock_rx) -> msg => {
                    self.run_cycle(&mut running)?;
                    debug!("clock: {}", msg.unwrap());
                },
            };

        };        

        Ok(())
    }

    fn fetch(&mut self) -> Result<Instruction> {
        let instr = Instruction::new(self.memory[self.pc as usize],self.memory[self.pc as usize + 1]);
        self.pc += 2;
        Ok(instr)
    }

    fn run_cycle(&mut self, running: &mut bool) -> Result<()>{
        let mut event_pump = self.sdl_context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    *running = false;
                }
                _ => {}
            }
        }
        let instr = self.fetch()?;
        debug!("execute: {:04X}, pc: {:04X}", instr.opcode, self.pc - 2);
        match instr.kind() {
            0x0 => {
                if instr.opcode == 0x00e0 {
                    self.video.clear();
                } else if instr.opcode == 0x00ee {
                    self.pc = self.stack.pop().unwrap(); // TODO: 需要后续编写错误处理
                }
            },
            0x1 => self.pc = instr.nnn(),
            0x2 => {
                self.stack.push(self.pc);
                self.pc = instr.nnn();
            },
            0x3 => {
                let x = self.registers[instr.x()];
                if x == instr.nn() as u8 {
                    self.pc += 2;
                }
            },
            0x4 => {
                let x = self.registers[instr.x()];
                if x != instr.nn() {
                    self.pc += 2;
                }
            },
            0x5 => {
                let x = self.registers[instr.x()];
                let y = self.registers[instr.y()];
                if x == y {
                    self.pc += 2;
                }
            },
            0x6 => {
                self.registers[instr.x()] = instr.nn();
            },
            0x7 => {
                self.registers[instr.x()] = self.registers[instr.x()].overflowing_add(instr.nn()).0;

            },
            0x8 => { //8XYN
                let x = self.registers[instr.x()];
                let y = self.registers[instr.y()];
                match instr.n() {
                    0 => self.registers[instr.x()] = y,
                    1 => self.registers[instr.x()] |= y,
                    2 => self.registers[instr.x()] &= y,
                    3 => self.registers[instr.x()] ^= y,
                    4 => { 
                        match x.overflowing_add(y) {
                            (n, false) => self.registers[instr.x()] = n,
                            (n, true) => {
                                self.registers[instr.x()] = n;
                                self.registers[0xf] = 1;
                            },
                        }
                    },
                    5 => { 
                        match x.overflowing_sub(y) {
                            (n, false) => {
                                self.registers[instr.x()] = n;
                                self.registers[0xf] = 1;
                            },
                            (n, true) => {
                                self.registers[instr.x()] = n;
                                self.registers[0xf] = 0;
                            },
                        }
                    },
                    7 => { 
                        match y.overflowing_sub(x) {
                            (n, false) => {
                                self.registers[instr.x()] = n;
                                self.registers[0xf] = 1;
                            },
                            (n, true) => {
                                self.registers[instr.x()] = n;
                                self.registers[0xf] = 0;
                            },
                        }
                    },
                    6 => { //ignore the y
                        self.registers[instr.x()] = x.shr(1);
                        self.registers[0xf] = x & 1;
                    }
                    0xe => { //ignore the y
                        self.registers[instr.x()] = x.shl(1);
                        self.registers[0xf] = x >> 7;                       
                    }
                    _ => (),
                }
            },
            0x9 => {
                let x = self.registers[instr.x()];
                let y = self.registers[instr.y()];
                if x != y {
                    self.pc += 2;
                }
            },
            0xA => {
                self.i = instr.nnn();
            },
            0xB => {
                self.pc = instr.nnn() + self.registers[0] as u16;
            },
            0xC => {
                let mut rng = rand::thread_rng();
                let r1: u8 = rng.gen();
                self.registers[instr.x()] = r1 & instr.nn();
            },
            0xD => {
                let x = (self.registers[instr.x()] % 64) as usize;
                let y = (self.registers[instr.y()] % 32) as usize;
                debug!("draw at: ({}, {})", x, y);
                let n = instr.n() as usize;
                self.registers[0xf] = self.video.flip(x, y, n, &self.memory[self.i as usize..self.i as usize + n])          
            },
            0xE => {
                let pressed_keys: HashSet<u8> = event_pump
                    .keyboard_state()
                    .pressed_scancodes()
                    .filter_map(|s| self.keyboard.scancode_to_key(&s))
                    .collect();
                let key = self.registers[instr.x()];
                let required_key_pressed = pressed_keys.contains(&key);
                match (required_key_pressed, instr.nn()) {
                    (true, 0x9E) => {
                        self.pc += 2;
                        info!("instr: {:04X}, key {:X?} pressed, key {:X?} required", instr.opcode, pressed_keys, key)
                    },
                    (false, 0xA1) => {
                        self.pc += 2;
                        info!("instr: {:04X}, key {:X?} pressed, key {:X?} not required", instr.opcode, pressed_keys, key)
                    },
                    _ => (),
                }
            },
            0xF => {
                let x = instr.x();
                match instr.nn() {
                    7 => self.registers[x] = self.delay_timer,
                    15 => self.delay_timer = self.registers[x],
                    18 => self.sound_timer = self.registers[x],
                    0x1E => self.i += self.registers[x] as u16,
                    0x33 => {
                        let mut x = self.registers[instr.x()];
                        self.memory[self.i as usize + 2] = x % 10;
                        x /= 10;
                        self.memory[self.i as usize + 1] = x % 10;
                        x /= 10;
                        self.memory[self.i as usize] = x;
                    },
                    0x0A => {
                        let pressed_keys: Vec<u8> = event_pump
                            .keyboard_state()
                            .pressed_scancodes()
                            .filter_map(|s| self.keyboard.scancode_to_key(&s))
                            .collect();
                        if pressed_keys.len() == 0 {
                            self.pc -= 2;
                        } else {
                            self.registers[instr.x()] = pressed_keys[0];
                            info!("key {:X} is being pressed", pressed_keys[0]);
                        }
                        
                    },
                    0x29 => {
                        let key = self.registers[instr.x()];
                        self.i = 0x50 + 5 * key as u16;
                        debug!("draw: {:X}", key);
                    }
                    0x55 => {
                        for n in 0..=0xf as usize {
                            self.memory[self.i as usize + n] = self.registers[n];
                        }
                    },
                    0x65 => {
                        for n in 0..=0xf as usize {
                            self.registers[n] = self.memory[self.i as usize + n];
                        }
                    },
                    _ => ()
                    
                }
            }
            _ => (),
        }
        Ok(())
    }
}

