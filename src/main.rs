pub mod audio;
pub mod font;
pub mod instruction;
pub mod keyboard;
pub mod machine;
pub mod rom;
pub mod video;

#[macro_use]
extern crate log;
extern crate clap;
extern crate sdl2;

use std::error::Error;
use std::result;

use clap::{App, Arg};

use machine::Machine;
use rom::ROM;

#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => { Err(Box::<dyn Error>::from(format!($($tt)*))) };
}

pub type Result<T> = result::Result<T, Box<dyn Error>>;

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
    machine.run()?;
    Ok(())
}
