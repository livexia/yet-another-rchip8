use std::result;
use std::error::Error;

use sdl2::{VideoSubsystem, render::WindowCanvas};

type Result<T> = result::Result<T, Box<dyn Error>>;

pub struct Video {
    sdl_video: VideoSubsystem,
    canvas: WindowCanvas,
    width: usize,
    height: usize,
    grid: Vec<Vec<u8>>,
}

impl Video {
    pub fn new(video_subsystem: VideoSubsystem, width: usize, height: usize) -> Result<Self> {
        let window = video_subsystem
            .window("yet-another-rchip8", 640, 320)
            .position_centered()
            .resizable()
            .build()?;
        let mut canvas = window
            .into_canvas()
            .accelerated()
            .build()?;
        canvas.set_logical_size(width as u32, height as u32)?;
        let grid = vec![vec![0; height]; width];
        Ok(Self {
            sdl_video: video_subsystem,
            canvas, width, height, grid
        })
    }

    pub fn draw(&mut self) -> Result<()>{
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(0, 0, 0 , 255));
        self.canvas.clear();
        self.canvas.set_draw_color(sdl2::pixels::Color::RGBA(255, 255, 255 , 255));
        for x in 0..64 {
            for y in 0..32 {
                if self.grid[x][y] != 0 {
                    self.canvas.draw_point((x as i32, y as i32))?;
                }
            }
        }
        self.canvas.present();
        Ok(())
    }

    pub fn flip(&mut self, x: usize, y: usize, n: usize, data: &[u8]) -> u8{
        let mut flag = 0;
        for offset_y in 0..n {
            let bits = data[offset_y];
            if y + offset_y == 32 {
                break;
            }
            for offset_x in 0..8 {
                if x + offset_x < 64 {
                    if self.grid[x + offset_x][y + offset_y] == 1 && bits >> (7 - offset_x) & 0x1 == 1 {
                        self.grid[x + offset_x][y + offset_y] = 0;
                        flag = 1;
                    } else if self.grid[x + offset_x][y + offset_y] == 0 && bits >> (7 - offset_x) & 0x1 == 1 {
                        self.grid[x + offset_x][y + offset_y] = 1;
                    }
                } else {
                    break;
                }
            }
        }
        flag
    }

    pub fn clear(&mut self) {
        self.grid = vec![vec![0; self.height]; self.width];
    }
}
