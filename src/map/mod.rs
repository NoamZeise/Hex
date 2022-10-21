use std::path::Path;

use tiled;
use crate::{TextDraw, GameObject, resource, camera::Camera, Colour};
use crate::{TextureManager, resource::Texture};
use sdl2::render::Canvas;
use sdl2::video::Window;
use geometry::Rect;

#[derive(Clone)]
struct Tile {
    pub tex : Texture,
    pub rect: Rect,
}

impl Tile {
    pub fn new() -> Tile {
        Tile {
            tex: Texture{ id: 0, width: 0, height: 0},
            rect: Rect::blank(),
        }
    }
}

fn load_tileset(tiles: &mut Vec<Tile>, ts: &tiled::Tileset, tex: resource::Texture) -> Result<(), String> {
    let mut id = ts.first_tile_id as usize;
    for y in 0..(ts.tile_count / ts.column_count) {
        for x in 0..ts.column_count {
            if id >= tiles.len() {
                return Err(String::from("Map Tile Count did not match actual tilecount"));
            }
            
            tiles[id].tex = tex;
            tiles[id].rect = Rect::new(
                ts.margin as f64 + ((ts.tile_width + ts.spacing)  * x) as f64,
                ts.margin as f64 + ((ts.tile_height + ts.spacing) * y) as f64,
                ts.tile_width as f64,
                ts.tile_height as f64,
            );
            id += 1;
        }
    }
    Ok(())
}

struct Layer {
    tile_draws: Vec<GameObject>,
}

impl Layer {
    fn new(l: &tiled::Layer, tiles: &Vec<Tile>) -> Layer {
        let mut layer = Layer { tile_draws: Vec::new() };
        for y in 0..l.height {
            for x in 0..l.width {
                let tile_id = l.tiles[(y * l.width + x) as usize] as usize;
                if tile_id == 0 { continue; }
                let tile = &tiles[tile_id];
                layer.tile_draws.push(
                    GameObject::new(
                        tile.tex,
                        Rect::new(
                            l.info.offset.x + (x as f64 * tile.rect.w),
                            l.info.offset.y + (y as f64 * tile.rect.h),
                            tile.rect.w,
                            tile.rect.h,
                        ),
                        tile.rect,
                        l.info.parallax,
                        Colour::new(
                            l.info.tint.r as u8,
                            l.info.tint.g as u8,
                            l.info.tint.b as u8,
                            (l.info.opacity * 255.0) as u8,
                        )
                    )
                );
            }
        }
        layer
    }
}

pub struct Map {
    tiled_map: tiled::Map,
    tiles : Vec<Tile>,
    layers : Vec<Layer>,
}

impl Map {
    pub fn new<'sdl, TexType>(filename: &str, tex_manager : &'sdl mut TextureManager<TexType>) -> Result<Self, String> {
        let mut map = Self {
            tiled_map: tiled::Map::new(filename).unwrap(),
            tiles: Vec::new(),
            layers: Vec::new(),
        };

        map.load_tilesets(tex_manager)?;
        map.set_map_draws();
        
        Ok(map)
    }

    pub fn draw(&self, cam: &mut Camera) {
        for l in self.layers.iter() {
            for t in l.tile_draws.iter() {
                cam.add_cam_space(t);
            }
        }
    }

    fn load_tilesets<'sdl, TexType>(&mut self, tex_manager : &'sdl mut TextureManager<TexType>) -> Result<(), String> {
        self.tiles.resize(self.tiled_map.total_tiles as usize, Tile::new());
        // blank tile
        self.tiles[0].rect.w = self.tiled_map.tile_width as f64;
        self.tiles[0].rect.h = self.tiled_map.tile_height as f64;
        for ts in self.tiled_map.tilesets.iter() {
            load_tileset(&mut self.tiles, ts, tex_manager.load(&Path::new(&ts.image_path))?)?;
        }
        Ok(())
    }

    fn set_map_draws(&mut self) {
        for l in self.tiled_map.layers.iter() {
            self.layers.push(Layer::new(&l, &self.tiles));
        }
    }
}
