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

const INITIAL_FALL_DELAY : f64 = 2.5;
const INITIAL_SPAWN_DELAY : f64 = 12.0;

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
    pub counted: bool,
}

impl Hex {
    pub fn blank() -> Hex {
        Hex {
            obj: GameObject::new_from_tex(resource::Texture { id: 0, width: 0, height: 0}),
            tile: Tile::Blank,
            rating: 0,
            to_kill: false,
            counted: false,
        }
    }
    pub fn new(obj: GameObject) -> Hex {
        Hex{
            obj,
            tile: Tile::Blank,
            rating: 0,
            to_kill: false,
            counted: false,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Tile {
    Red,
    Green,
    Blue,
    Yellow,
    Blank,
}

fn random_tile() -> Tile {
    let x: usize = rand::thread_rng().gen_range(0..4);
    match x {
        0 => Tile::Green,
        1 => Tile::Red,
        2 => Tile::Blue,
        3 => Tile::Yellow,
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

    score: usize,
    black_hex: Texture,
    white_hex: Texture,
    lost: bool,
}

impl HexGrid {
    pub fn new<'sdl , TexType>(tm: &mut TextureManager<'sdl, TexType>) -> Result<HexGrid, String> {
        let mut tiles = HashMap::<Tile, Texture>::new();
        tiles.insert(Tile::Blank, tm.load(Path::new("textures/tile/blank.png"))?);
        tiles.insert(Tile::Red, tm.load(Path::new("textures/tile/red.png"))?);
        tiles.insert(Tile::Green, tm.load(Path::new("textures/tile/green.png"))?);
        tiles.insert(Tile::Blue, tm.load(Path::new("textures/tile/blue.png"))?);
        tiles.insert(Tile::Yellow, tm.load(Path::new("textures/tile/yellow.png"))?);
        
        let obj = GameObject::new_from_tex(tm.load(Path::new("textures/tile/blue.png"))?);
        let black_hex = tm.load(Path::new("textures/tile/mid.png"))?;
        let white_hex = tm.load(Path::new("textures/tile/white.png"))?;
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
                    grid[0].obj.texture = black_hex;
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
            drop_delay: INITIAL_FALL_DELAY,
            drop_timer: 0.0,
            spawn_timer: INITIAL_SPAWN_DELAY / 2.0,
            spawn_delay: INITIAL_SPAWN_DELAY,
            score: 0,
            black_hex,
            white_hex,
            lost: false,
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

    fn move_ring(&mut self, y: usize, out: bool) -> bool {
        let mut moved = false;
        if (out && y == BOARD_RADIUS - 1) || (!out && y == 1) { return moved; }
        let mut i = 0;
        while i < get_y_size(y) {
            let change_x = if out { i + (i / y) } else { i - (i / y) };
            let change_y = if out { y + 1} else { y - 1};
            if self.get_tile(i, y) != Tile::Blank &&
                self.get_tile(change_x, change_y) == Tile::Blank{
                    moved = true;
                self.change_tile(change_x, change_y, self.get_tile(i, y));
                self.change_tile(i, y, Tile::Blank);
            }
            
            i+=1;
        };
        moved
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
            //self.move_ring(self.hl_y, true);
            self.drop_timer = self.drop_delay;
        }
         if input.b && !self.prev_input.b {
             //self.move_ring(self.hl_y, false);
         }

         if input.debug_1 && !self.prev_input.debug_1 {
             self.score += 10;
        }

        self.prev_input = *input;
    }

    fn spawn_ring(&mut self) {
        let mut prev = Tile::Blank;
        for x in 0..6 {
            if self.get_tile(x, 1) != Tile::Blank {
                self.lost = true;
                self.grid[Self::get_index(x, 1)].obj.texture = self.white_hex;
            }  else {
                let mut tile = random_tile();
                if tile == prev {
                    tile = random_tile();
                }
                self.change_tile(x, 1, tile);
                prev = tile;
            }
        }
    }

    fn per_neighbour(&mut self, n: usize, x: usize, y: usize, t: Tile, f: fn(&mut Self, usize, usize, usize, Tile) -> usize) -> usize {
        //check l/r
        let mut n = n;
        n = f(self, n, x + 1, y, t);
        n = f(self, n, if x == 0 { get_y_size(y) - 1} else {x  - 1}, y, t);
        
        if y != BOARD_RADIUS - 1 {
            
            let outer = (x as f64) / (y as f64);
            if outer.fract() < 0.1 {
                let outer = outer as usize;
                n = f(self, n, x + outer, y + 1, t);
                n =  f(self, n, x + outer + 1, y + 1, t);
                let x = if outer == 0 && x == 0 {x + get_y_size(y + 1) } else { x };
                n = f(self, n, x + outer - 1, y + 1, t);

            } else {
               n =  f(self, n, (x as f64 + outer).floor() as usize, y + 1, t);
               n =  f(self, n, (x as f64 + outer).ceil() as usize, y + 1, t);
            }
        }

        if y > 1 {
            let inner = (x as f64) / (y as f64);
            if inner.fract() < 0.1 {
                let outer = inner as usize;
               n =  f(self, n, x - outer, y - 1, t);

            } else {
               n =  f(self, n, (x as f64 - inner).floor() as usize, y - 1, t);
               n =  f(self, n, (x as f64 - inner).ceil() as usize, y - 1, t);
            }
        }
        n
    }
    fn nb(&mut self, n: usize, x: usize, y: usize, t: Tile) -> usize {
        let i = Self::get_index(x, y);
        if self.grid[i].tile == t && !self.grid[i].counted {
            self.grid[i].counted = true;
            return self.per_neighbour(n + 1, x, y, t, Self::nb);
        }
        
        n
    }

    fn all_count(&mut self, x: usize, y: usize, t: Tile) {
        let i = Self::get_index(x, y);
        if !self.grid[i].counted {
            self.grid[i].rating = self.per_neighbour(0, x, y, t, Self::nb) as i32;
            self.grid[i].counted = true;
        }
    }

    fn nhbr_clear(&mut self, _: usize, o_x: usize, o_y: usize, t : Tile) -> usize {
        let i = Self::get_index(o_x, o_y);
        if self.grid[i].tile == t && ! self.grid[i].to_kill {
            self.grid[Self::get_index(o_x, o_y)].to_kill = true;
            self.per_neighbour(0, o_x, o_y, t, Self::nhbr_clear); 
        }
        0
    }

    fn step_clear(&mut self, x: usize, y: usize, t: Tile) {
        //if y == 1 { return; }
        let i = Self::get_index(x, y);
        if self.grid[i].rating > 4   {
            self.grid[i].to_kill = true;
            self.per_neighbour(0, x, y, t, Self::nhbr_clear); 
        }
    }

    fn wipe_killed(&mut self, x:usize, y:usize, _:Tile) {
        let i = Self::get_index(x, y);
        if self.grid[i].to_kill {
            self.change_tile(x, y, Tile::Blank);
            self.grid[i].obj.texture = self.black_hex;
            self.grid[i].to_kill = false;
            self.score += 1;
        }
    }
        

    fn iter_grid(&mut self,  f: fn(&mut Self, usize, usize, Tile) -> ()) {
        for y in 0..BOARD_RADIUS {
            for x in 0..get_y_size(y) {
                let t = self.get_tile(x, y);
               if t != Tile::Blank {
                    f(self, x ,y, t);
                }
            }
        }
    }

    fn clear_lines(&mut self) {
        self.iter_grid(|s: &mut Self, x: usize, y: usize, _:Tile| {
            let i = Self::get_index(x, y);
            s.grid[i].counted = false;
            s.grid[i].rating = 0;
        });
        self.iter_grid(Self::all_count);
        self.iter_grid(Self::step_clear);
        self.iter_grid(Self::wipe_killed);   
    }

    fn game_logic(&mut self, t: f64) {
        self.drop_timer += t;
        if self.drop_timer > self.drop_delay {
            self.drop_delay = INITIAL_FALL_DELAY - (self.score as f64 / 60.0).powf(0.5);
            self.drop_timer = 0.0;
            let mut y = BOARD_RADIUS;
            let mut moved = false;
            while y > 1 {
                moved |= self.move_ring(y - 1, true);
                y -= 1;
            }
            let mut i = 1;
            while i < self.grid.len() {
                self.grid[i].obj.texture = self.tiles[&self.grid[i].tile];
                i += 1;
            }
            if !moved {
                self.spawn_ring();
            } 
        }

        self.clear_lines();
    }

    pub fn lost(&self) -> bool {
        self.lost
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn reset(&mut self) {
        self.spawn_timer = INITIAL_SPAWN_DELAY / 2.0;
        self.spawn_delay = INITIAL_SPAWN_DELAY;
        self.drop_delay = INITIAL_FALL_DELAY;
        self.score = 0;
        self.lost = false;
        self.hl_y = 1;
        self.iter_grid(|s: &mut Self, x: usize, y: usize, _: Tile| {
            s.change_tile(x, y, Tile::Blank);
        });
    }

    pub fn spawn_ratio(&self) -> f64 {
        self.drop_delay as f64/ self.drop_timer as f64
    }
}
