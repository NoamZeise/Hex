use crate::Colour;
use crate::resource::Texture;
use crate::{GameObject, camera::Camera, TextureManager, resource, input::Input};
use geometry::*;
use std::path::Path;
use std::f64::consts;
use std::collections::HashMap;

const fn board_size(r: usize) -> usize {
    let mut sum = 1;
    let mut i = 0;
    while i < r {
        sum += 6 * i;
        i+=1;
    }
    sum
}

const HEX: Rect = Rect{x: 11.0, y: 14.0, w: 16.0, h: 16.0};
const BOARD_CENTER: Vec2 = Vec2{x: 120.0, y: 80.0};
const BOARD_RADIUS : usize = 5;
const BOARD_SIZE : usize = board_size(BOARD_RADIUS);
const HL_SWAP : f64 = 0.4;

fn get_y_size(y : usize) -> usize {
    if y == 0 { 1 } else { y * 6 }
}


fn board_pos(x: usize, y: usize) -> Vec2 {
    let mut pos = Vec2::new(BOARD_CENTER.x - HEX.w / 2.0, BOARD_CENTER.y - HEX.h / 2.0);
    
    let angle = if y != 0 {
        ((x as f64 / y as f64)/6.0) * consts::PI * 2.0 - consts::FRAC_PI_2
    } else {
        0.0
    };
    
    
    let lim = consts::FRAC_PI_6 * 5.0 + consts::FRAC_PI_2;
    let dir = Vec2::new(angle.cos(), angle.sin());

    //println!("angle: {}   x vec: {}  y vec: {}", angle, dir.x, dir.y);
    let mut x_diff = (dir.x * (HEX.x) * (y as f64)).round();
    if x_diff < 0.0 {
        x_diff = (x_diff / (HEX.x)).floor() * (HEX.x);
    } else {
        x_diff = (x_diff / (HEX.x)).ceil() * (HEX.x);
    }

    let mut y_diff = (dir.y * (HEX.y) * (y as f64)).round();
    if y_diff < 0.0 {
        y_diff = (y_diff / (HEX.y / 2.0)).ceil() * (HEX.y / 2.0);
    } else {
        y_diff = (y_diff / (HEX.y / 2.0)).floor() * (HEX.y / 2.0);
    }


    pos.x += x_diff;
    pos.y += y_diff;

    

    //println!("{}   {}", pos.x, pos.y);

    Vec2::new(pos.x, pos.y)
}

#[derive(Clone, Copy)]
struct Hex {
    pub tile: Tile,
    pub obj: GameObject,
}

impl Hex {
    pub fn blank() -> Hex {
        Hex {
            obj: GameObject::new_from_tex(resource::Texture { id: 0, width: 0, height: 0}),
            tile: Tile::Blank,
        }
    }
    pub fn new(obj: GameObject) -> Hex {
        Hex{
            obj,
            tile: Tile::Blank,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Red,
    Green,
    Blue,
    Blank,
}

pub struct HexGrid {
    grid : [Hex; BOARD_SIZE],
    tiles: HashMap<Tile, Texture>,
    hl: [GameObject; 2],
    hl_y: usize,
    hl_timer: f64,
    hl_active: usize,
    prev_input: Input,
}

impl HexGrid {
    pub fn new<'sdl , TexType>(tm: &mut TextureManager<'sdl, TexType>) -> Result<HexGrid, String> {
        let mut tiles = HashMap::<Tile, Texture>::new();
        tiles.insert(Tile::Blank, tm.load(Path::new("textures/tile/blank.png"))?);
        tiles.insert(Tile::Red, tm.load(Path::new("textures/tile/red.png"))?);
        tiles.insert(Tile::Green, tm.load(Path::new("textures/tile/green.png"))?);
        tiles.insert(Tile::Blue, tm.load(Path::new("textures/tile/blue.png"))?);
        
        let obj = GameObject::new_from_tex(tm.load(Path::new("textures/tile/blue.png"))?);
        let mut mid_tex = GameObject::new_from_tex(tm.load(Path::new("textures/mid.png"))?);
        
        let mut grid = [Hex::blank();BOARD_SIZE];
        let mut off = BOARD_CENTER;
        let mut x_total = 0;
        for y in 0..BOARD_RADIUS {
            let x_size = if y == 0 {
                1
            } else {
                y * 6
            };
            for x in 0..x_size {
                let mut obj = obj;
                let pos = board_pos(x, y);
                obj.rect.x = pos.x;
                obj.rect.y = pos.y;
                if y == 0 && x == 0 {
                    mid_tex.rect.x = obj.rect.x - ((mid_tex.rect.w - obj.rect.w)/2.0);
                    mid_tex.rect.y = obj.rect.y - ((mid_tex.rect.h - obj.rect.h)/2.0);
                }
                grid[x_total + x] = Hex::new(obj);
                if x == 0 {
                    grid[x_total + x].tile = Tile::Blue;
                }
                grid[x_total + x].obj.texture = tiles[&grid[x_total + x].tile];
            }
            off.y += obj.rect.h - 1.0;
            x_total += x_size;
        }

        Ok(HexGrid {
            grid,
            tiles,
            hl: [GameObject::new_from_tex(tm.load(Path::new("textures/hl1.png"))?),
                 GameObject::new_from_tex(tm.load(Path::new("textures/hl2.png"))?)],
            hl_y: 0,
            hl_timer: 0.0,
            hl_active: 0,
            prev_input: Input::new(),
        })
    }

    pub fn draw(&self, cam: &mut Camera) {
        for g in 0..BOARD_SIZE {
            cam.add_cam_space(&self.grid[g].obj);
        }
        let mut active = self.hl[self.hl_active];
        for x in 0..self.y_ring() {
            active.rect = self.grid[Self::get_index(x, self.hl_y)].obj.rect;
            cam.add_cam_space(&active);
        }
    }
    
    fn get_index(x: usize, y: usize) -> usize {
        let off = if y == 0 {
            0
        } else {
            board_size(y)
        };
        off + x
    }

    pub fn update(&mut self, timer: &f64, input: &Input) {
        self.input_handle(input);
        //self.grid[Self::get_index(0, 1)].obj.colour = Colour::new(10, 0, 255, 255);
 
        self.hl_timer += timer;
        if self.hl_timer > HL_SWAP {
            self.hl_timer = 0.0;
            self.hl_active = (self.hl_active + 1) % 2;
        }
    }

    fn y_ring(&self) -> usize {
        get_y_size(self.hl_y)
    }

    fn get_tile(&self, x: usize, y: usize) -> Tile {
        self.grid[Self::get_index(x, self.hl_y)].tile
    }

    fn change_tile(&mut self, x: usize, y: usize, tile: Tile) {
        let i = Self::get_index(x, y);
        self.grid[i].tile = tile;
        self.grid[i].obj.texture = self.tiles[&tile];
    }

    fn ring_shift(&mut self, dir: i32) {
        if dir.signum() == 1 {
        let mut last = self.grid[Self::get_index(self.y_ring() - 1, self.hl_y)].tile;
        let mut i = 0;
        while i < self.y_ring() {
            let tmp = self.get_tile(i, self.hl_y);
            self.change_tile(i, self.hl_y, last);
            last = tmp;

            i+=1;
        }
        }
        else {
            let mut last = self.grid[Self::get_index(0, self.hl_y)].tile;
            let mut i = self.y_ring();
            while i > 0 {
                let tmp = self.get_tile(i - 1, self.hl_y);
                self.change_tile(i - 1, self.hl_y, last);
                last = tmp;
                
                i-=1;
            }
        }
    }
    
    fn input_handle(&mut self, input: &Input) {
        if input.up && !self.prev_input.up {
            self.hl_y = (self.hl_y + 1) % BOARD_RADIUS;
            if self.hl_y == 0 {
                self.hl_y += 1;
            }
        }
        if input.down && !self.prev_input.down {
            self.hl_y = ((self.hl_y + BOARD_RADIUS) - 1) % BOARD_RADIUS;
            if self.hl_y == 0 {
                self.hl_y = BOARD_RADIUS - 1;
            }
        }

        if input.right && !self.prev_input.right {
            self.ring_shift(1);
        }
        
        if input.left && !self.prev_input.left {
            self.ring_shift(-1);
        }

        self.prev_input = *input;
    }

}
