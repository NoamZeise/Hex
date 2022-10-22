use crate::{GameObject, camera::Camera, TextureManager, resource};
use geometry::*;
use std::path::Path;
use std::f64::consts;

const fn board_size(r: usize) -> usize {
    let mut sum = 0;
    let mut i = 0;
    while i < r {
        sum += 6_usize.pow(i as u32);
        i+=1;
    }
    sum
}

const HEX: Rect = Rect{x: 11.0, y: 7.0, w: 16.0, h: 16.0};
const BOARD_CENTER: Vec2 = Vec2{x: 120.0, y: 80.0};
const BOARD_RADIUS : usize = 3;
const BOARD_SIZE : usize = board_size(BOARD_RADIUS);


fn board_pos(x: usize, y: usize) -> Vec2 {
    let mut pos = Vec2::new(BOARD_CENTER.x - HEX.w / 2.0, BOARD_CENTER.y - HEX.h / 2.0);
    let rad = 6_i32.pow(y as u32);

    let angle = (((x as f64 / y as f64) - 3.0) / 3.0) * consts::PI;

    let dir = Vec2::new(angle.cos() * y as f64, angle.sin() * y as f64);


    pos.x += dir.x * HEX.x;
    pos.y += dir.y * HEX.h;

    Vec2::new(pos.x, pos.y)
}

#[derive(Clone, Copy)]
struct Hex {
    pub obj: GameObject,
}

impl Hex {
    pub fn blank() -> Hex {
        Hex {
            obj: GameObject::new_from_tex(resource::Texture { id: 0, width: 0, height: 0}),
        }
    }
    pub fn new(obj: GameObject) -> Hex {
        Hex{
            obj,
        }
    }
}

pub struct HexGrid {
    grid : [Hex; BOARD_SIZE],
    mid_tex: GameObject,
}

impl HexGrid {
    pub fn new<'sdl , TexType>(tm: &mut TextureManager<'sdl, TexType>) -> Result<HexGrid, String> {
        let obj = GameObject::new_from_tex(tm.load(Path::new("textures/hexagon.png"))?);
        let mut mid_tex = GameObject::new_from_tex(tm.load(Path::new("textures/mid.png"))?);
        
        let mut grid = [Hex::blank();BOARD_SIZE];
        let mut off = BOARD_CENTER;
        let mut x_total = 0;
        for y in 0..BOARD_RADIUS {
            let x_size = 6_usize.pow(y as u32);
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
            }
            off.y += obj.rect.h - 1.0;
            x_total += x_size;
        }
        Ok(HexGrid {
            grid,
            mid_tex,
        })
    }

    pub fn draw(&self, cam: &mut Camera) {
        for g in 0..BOARD_SIZE {
            cam.add_cam_space(&self.grid[g].obj);
        }
        cam.add_cam_space(&self.mid_tex);
    }

}
