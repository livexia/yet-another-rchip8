use rand::thread_rng;
use rand::Rng;
use sdl2::audio::AudioCallback;
use sdl2::audio::AudioDevice;
use sdl2::audio::AudioSpecDesired;
use sdl2::AudioSubsystem;

use crate::{audio::AudioPlay, Result};

#[allow(dead_code)]
pub struct Sdl2Audio {
    sdl_audio: AudioSubsystem,
    device: AudioDevice<MyCallback>,
}

impl Sdl2Audio {
    pub fn new(audio_subsystem: AudioSubsystem) -> Result<Self> {
        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        // None: use default device
        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // Show obtained AudioSpec
            info!("{:?}", spec);
            MyCallback { volume: 0.1 }
        })?;

        Ok(Self {
            sdl_audio: audio_subsystem,
            device,
        })
    }
}

impl AudioPlay for Sdl2Audio {
    fn resume(&self) {
        self.device.resume()
    }

    fn pause(&self) {
        self.device.pause()
    }
}

struct MyCallback {
    volume: f32,
}

impl AudioCallback for MyCallback {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut rng = thread_rng();

        // Generate white noise
        for x in out.iter_mut() {
            *x = (rng.gen_range(0.0..2.0) - 1.0) * self.volume; //TODO: white noise to beeps
        }
    }
}
