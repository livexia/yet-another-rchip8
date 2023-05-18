pub mod audio;
pub mod font;
pub mod instruction;
pub mod keyboard;
pub mod machine;
pub mod rom;
pub mod sdl2_audio;
pub mod video;

#[macro_use]
extern crate log;
extern crate clap;
extern crate sdl2;

use chrono::{DateTime, Utc};
use crossbeam_channel::{select, unbounded, Sender};
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{event::Event, EventPump};
use sdl2_audio::Sdl2Audio;
use std::collections::HashMap;
use std::error::Error;
use std::result;
use std::thread;
use std::time::Duration;

use clap::{App, Arg};

use machine::Machine;
use rom::ROM;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) };
}

pub type Result<T> = result::Result<T, Box<dyn Error>>;

pub struct Sdl2KeyMap {
    scancodes_map: HashMap<Scancode, u8>,
}

impl Sdl2KeyMap {
    pub fn new(layout: &HashMap<Scancode, u8>) -> Result<Self> {
        let scancodes_map = layout.clone();
        if layout.len() != 16 {
            return err!("layout will not be matched, the layout length is not 16");
        }
        Ok(Sdl2KeyMap { scancodes_map })
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

impl Default for Sdl2KeyMap {
    fn default() -> Self {
        Self::new(&Self::default_keyboard_layout()).unwrap()
    }
}

fn sdl2_key_event(
    machine: &mut Machine<Sdl2Audio>,
    running: &mut bool,
    event_pump: &mut EventPump,
    key_map: &Sdl2KeyMap,
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
                    machine.key_down(key);
                    debug!("KeyDown: {:?} -> {}", scancode, key);
                }
            }
            Event::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(key) = key_map.scancode_to_key(&scancode) {
                    machine.key_up(key);
                    debug!("KeyUp: {:?} -> {}", scancode, key);
                }
            }
            _ => {}
        }
    }
}

fn sdl2_draw(canvas: &mut Canvas<Window>, machine: &Machine<Sdl2Audio>) -> Result<()> {
    let grid = machine.get_display();
    for (x, row) in grid.iter().enumerate() {
        for (y, &item) in row.iter().enumerate() {
            if item != 0 {
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 255, 255));
            } else {
                canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 255));
            }
            canvas.draw_point((x as i32, y as i32))?;
        }
    }
    canvas.present();
    Ok(())
}

fn sdl2_init(width: u32, height: u32) -> Result<(Canvas<Window>, Sdl2Audio, EventPump)> {
    let sdl_context = sdl2::init()?;

    let video = sdl_context.video()?;
    let window = video
        .window("yet-another-rchip8", 640, 320)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    canvas.set_logical_size(width, height)?;

    let audio = Sdl2Audio::new(sdl_context.audio()?)?;
    Ok((canvas, audio, sdl_context.event_pump()?))
}

fn sdl2_emulate(machine: &mut Machine<Sdl2Audio>) -> Result<()> {
    let (timer_tx, timer_rx) = unbounded();
    let (clock_tx, clock_rx) = unbounded();

    // timer 60Hz ~= 16667 micros
    // clock 500Hz ~= 2000 micros
    sender(timer_tx, clock_tx, 60, 500);

    let (width, height) = (machine.width(), machine.height());
    let (mut canvas, audio, mut event_pump) = sdl2_init(width as u32, height as u32)?;
    machine.init_sound(audio);

    let key_map = Sdl2KeyMap::default();

    let mut running = true;
    while running && !machine.is_halt() {
        select! {
            recv(timer_rx) -> msg => {
                machine.update_timer();
                sdl2_draw(&mut canvas, machine)?;
                debug!("timer: {}", msg.unwrap());
            },
            recv(clock_rx) -> msg => {
                sdl2_key_event(machine, &mut running, &mut event_pump, &key_map);
                machine.run_cycle()?;
                debug!("clock: {}", msg.unwrap());
            },
        };
    }
    Ok(())
}

fn sender(
    timer_tx: Sender<DateTime<Utc>>,
    clock_tx: Sender<DateTime<Utc>>,
    timer_freq: u64,
    clock_freq: u64,
) {
    let timer_dur = Duration::from_micros(1000000 / timer_freq);
    thread::spawn(move || loop {
        thread::sleep(timer_dur);
        let _ = timer_tx.send(chrono::Utc::now());
    });
    let clock_dur = Duration::from_micros(1000000 / clock_freq);
    thread::spawn(move || loop {
        thread::sleep(clock_dur);
        let _ = clock_tx.send(chrono::Utc::now());
    });
}

fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("yet-another-rchip8")
        .version("0.0001")
        .author("livexia")
        .arg(
            Arg::with_name("ROM")
                .short("r")
                .long("rom")
                .takes_value(true)
                .help("Sets the rom file to load"),
        )
        .get_matches();

    let rom = matches.value_of("ROM").unwrap_or("IBM_Logo.hex");
    let rom = ROM::new(rom)?;
    let mut machine = Machine::new()?;
    machine.load_font()?;
    machine.load_rom(&rom)?;
    sdl2_emulate(&mut machine)?;
    Ok(())
}
