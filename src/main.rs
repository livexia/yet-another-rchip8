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

use crossbeam_channel::{select, unbounded};
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

pub struct KeyMap {
    scancodes_map: HashMap<Scancode, u8>,
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

impl Default for KeyMap {
    fn default() -> Self {
        Self::new(&Self::default_keyboard_layout()).unwrap()
    }
}

fn process_key_event(
    machine: &mut Machine<Sdl2Audio>,
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
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0, 255));
    canvas.clear();
    canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 255, 255));
    let grid = machine.get_display();
    for (x, row) in grid.iter().enumerate() {
        for (y, &item) in row.iter().enumerate() {
            if item != 0 {
                canvas.draw_point((x as i32, y as i32))?;
            }
        }
    }
    canvas.present();
    Ok(())
}

fn emulate(machine: &mut Machine<Sdl2Audio>) -> Result<()> {
    let (timer_tx, timer_rx) = unbounded();
    let (clock_tx, clock_rx) = unbounded();

    // timer 60Hz ~= 16667 micros
    let timer_dur = Duration::from_micros(1000000 / 60);
    thread::spawn(move || loop {
        thread::sleep(timer_dur);
        timer_tx.send(chrono::Utc::now()).unwrap()
    });
    // clock 500Hz ~= 2000 micros
    let clock_dur = Duration::from_micros(1000000 / 500);
    thread::spawn(move || loop {
        thread::sleep(clock_dur);
        clock_tx.send(chrono::Utc::now()).unwrap();
    });

    let mut running = true;
    let (width, height) = (machine.width(), machine.height());

    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;
    let audio = Sdl2Audio::new(sdl_context.audio()?)?;
    let window = video
        .window("yet-another-rchip8", 640, 320)
        .position_centered()
        .resizable()
        .build()?;
    let mut canvas = window.into_canvas().accelerated().build()?;
    canvas.set_logical_size(width as u32, height as u32)?;
    machine.init_sound(audio);

    let mut event_pump = sdl_context.event_pump()?;
    let key_map = KeyMap::default();
    while running && !machine.is_halt() {
        select! {
            recv(timer_rx) -> msg => {
                machine.decrement_delay_timer();
                machine.decrement_sound_timer();
                sdl2_draw(&mut canvas, machine)?;
                debug!("timer: {}", msg.unwrap());
            },
            recv(clock_rx) -> msg => {
                process_key_event(machine, &mut running, &mut event_pump, &key_map);
                machine.run_cycle()?;
                debug!("clock: {}", msg.unwrap());
            },
        };
    }
    Ok(())
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
    emulate(&mut machine)?;
    Ok(())
}
