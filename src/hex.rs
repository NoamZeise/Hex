use crate::Colour;
use crate::resource::Texture;
use crate::{GameObject, camera::Camera, TextureManager, resource, input::Input};
use geometry::*;
use std::path::Path;
use std::f64::consts;
use std::collections::HashMap;

use rand::Rng;


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
const BOARD_RADIUS : usize = 6;
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
    
    let dir = Vec2::new(angle.cos(), angle.sin());

    let mut x_diff = (dir.x * (HEX.x) * (y as f64)).round();
    if x_diff < 0.0 {
        x_diff = (x_diff / (HEX.x)).floor() * (HEX.x);
    } else {
        x_diff = (x_diff / (HEX.x)).ceil() * (HEX.x);
    }

    let mut y_diff = ((dir.y) * (HEX.y) * (y as f64)).round();
    if y_diff < 0.0 {
        y_diff = (y_diff / (HEX.y / 2.0)).ceil() * (HEX.y / 2.0);
    } else {
        y_diff = (y_diff / (HEX.y / 2.0)).floor() * (HEX.y / 2.0);
    }

    if y == 5 {
        //im ashamed
        match x {
            2 | 3 | 27 | 28 => y_diff += HEX.y / 2.0,

            12 | 13 | 17 | 18 => y_diff -= HEX.y / 2.0,

            _ => (),
        }
    }

    pos.x += x_diff;
    pos.y += y_diff;

    Vec2::new(pos.x, pos.y)
}

#[derive(Clone, Copy)]
struct Hex {
    pub tile: Tile,
    pub obj: GameObject,
    pub rating: i32,
    pub to_kill: bool,
}

impl Hex {
    pub fn blank() -> Hex {
        Hex {
            obj: GameObject::new_from_tex(resource::Texture { id: 0, width: 0, height: 0}),
            tile: Tile::Blank,
            rating: 0,
            to_kill: false,
        }
    }
    pub fn new(obj: GameObject) -> Hex {
        Hex{
            obj,
            tile: Tile::Blank,
            rating: 0,
            to_kill: false,
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

fn random_tile() -> Tile {
    let x: usize = rand::thread_rng().gen_range(0..3);
    match x {
        0 => Tile::Green,
        1 => Tile::Red,
        2 => Tile::Blue,
        _ => Tile::Blank,
    }
}

pub struct HexGrid {
    grid : [Hex; BOARD_SIZE],
    tiles: HashMap<Tile, Texture>,
    hl: [GameObject; 2],
    hl_y: usize,
    hl_timer: f64,
    hl_active: usize,
    prev_input: Input,

    drop_delay: f64,
    drop_timer: f64,
    
    spawn_timer: f64,
    spawn_delay: f64,

    score: f64,
}

impl HexGrid {
    pub fn new<'sdl , TexType>(tm: &mut TextureManager<'sdl, TexType>) -> Result<HexGrid, String> {
        let mut tiles = HashMap::<Tile, Texture>::new();
        tiles.insert(Tile::Blank, tm.load(Path::new("textures/tile/blank.png"))?);
        tiles.insert(Tile::Red, tm.load(Path::new("textures/tile/red.png"))?);
        tiles.insert(Tile::Green, tm.load(Path::new("textures/tile/green.png"))?);
        tiles.insert(Tile::Blue, tm.load(Path::new("textures/tile/blue.png"))?);
        
        let obj = GameObject::new_from_tex(tm.load(Path::new("textures/tile/blue.png"))?);
        
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
                grid[x_total + x] = Hex::new(obj);
                //if x == 0 {
                //    grid[x_total + x].tile = Tile::Blue;
                //}
                grid[x_total + x].obj.texture = tiles[&grid[x_total + x].tile];
                if x_total + x == 0 {
                    grid[0].obj.texture = tm.load(Path::new("textures/tile/mid.png"))?;
                }
            }
            off.y += obj.rect.h - 1.0;
            x_total += x_size;
        }

        Ok(HexGrid {
            grid,
            tiles,
            hl: [GameObject::new_from_tex(tm.load(Path::new("textures/hl1.png"))?),
                 GameObject::new_from_tex(tm.load(Path::new("textures/hl2.png"))?)],
            hl_y: 1,
            hl_timer: 0.0,
            hl_active: 0,
            prev_input: Input::new(),
            drop_delay: 3.0,
            drop_timer: 0.0,
            spawn_timer: 0.0,
            spawn_delay: 12.0,
            score: 0.0,
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
    
    fn get_index(mut x: usize, y: usize) -> usize {
        let off = if y == 0 {
            return 0;
        } else {
            board_size(y)
        };
        x = x % get_y_size(y);
        off + x
    }

    pub fn update(&mut self, timer: &f64, input: &Input) {
        self.input_handle(input);
        self.game_logic(*timer);
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
        self.grid[Self::get_index(x, y)].tile
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

    fn move_ring(&mut self, y: usize, out: bool) {
        if (out && y == BOARD_RADIUS - 1) || (!out && y == 1) { return; }
        let mut i = 0;
        while i < get_y_size(y) {
            let change_x = if out { i + (i / y) } else { i - (i / y) };
            let change_y = if out { y + 1} else { y - 1};
            if self.get_tile(i, y) != Tile::Blank &&
               self.get_tile(change_x, change_y) == Tile::Blank{
                self.change_tile(change_x, change_y, self.get_tile(i, y));
                self.change_tile(i, y, Tile::Blank);
            }
            
            i+=1;
        };
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

        if input.a && !self.prev_input.a {
            self.move_ring(self.hl_y, true);
        }
         if input.b && !self.prev_input.b {
            self.move_ring(self.hl_y, false);
        }

        self.prev_input = *input;
    }

    fn spawn_ring(&mut self) {
        for x in 0..6 {
            self.change_tile(x, 1, random_tile());
        }
    }

    fn clear_lines(&mut self) {
        for y in 0..BOARD_RADIUS {
            for x in 0..get_y_size(y) {
                let t = self.get_tile(x, y);
                if t != Tile::Blank {
                    
                }
            }
        }
    }

    fn game_logic(&mut self, t: f64) {

        self.drop_timer += t;
        if self.drop_timer > self.drop_delay {
            self.drop_timer = 0.0;
            let mut y = BOARD_RADIUS;
            while y > 1 {
                self.move_ring(y - 1, true);
                y -= 1;
            }
        }

        self.spawn_timer += t;
        if self.spawn_timer > self.spawn_delay {
            self.spawn_timer = 0.0;
            self.spawn_ring();
        }

        self.clear_lines();
    }

}
