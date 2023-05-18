#[allow(dead_code)]
pub struct Video {
    width: usize,
    height: usize,
    grid: Vec<Vec<u8>>,
}

impl Video {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![vec![0; height]; width];
        Self {
            width,
            height,
            grid,
        }
    }

    pub fn draw(&mut self, x: usize, y: usize, n: usize, data: &[u8]) -> u8 {
        let mut flag = 0;
        for (offset_y, bits) in data.iter().enumerate().take(n) {
            let new_y = y + offset_y;
            if new_y == 32 {
                break;
            }
            for offset_x in 0..8 {
                let new_x = x + offset_x;
                if new_x < 64 {
                    if self.grid[new_x][new_y] == 1 && (bits >> (7 - offset_x)) & 0x1 == 1 {
                        self.grid[new_x][new_y] = 0;
                        flag = 1;
                    } else if self.grid[new_x][new_y] == 0 && (bits >> (7 - offset_x)) & 0x1 == 1 {
                        self.grid[new_x][new_y] = 1;
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

    pub fn get_grid(&self) -> &[Vec<u8>] {
        &self.grid
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
