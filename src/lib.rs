use sdl2::controller::GameController;
use sdl2::render::{TextureCreator, Texture, Canvas};
use sdl2::video::Window;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::ttf;

use std::collections::HashMap;
use std::path::Path;
use std::clone::Clone;

pub mod input;
use geometry::*;
pub mod map;
pub mod camera;

trait RectConversion {
    fn new_from_sdl_rect(sdl_rect : &sdl2::rect::Rect) -> Self;
    fn to_sdl_rect(&self) -> sdl2::rect::Rect;
}

impl RectConversion for Rect{
    /// Use an `sdl2::rect::Rect` to construct a `Rect`
    fn new_from_sdl_rect(sdl_rect : &sdl2::rect::Rect) -> Self {
        Rect {
            x: sdl_rect.x as f64,
            y: sdl_rect.y as f64,
            w: sdl_rect.w as f64,
            h: sdl_rect.h as f64
        }
    }
    
    /// construct an `sdl2::rect::Rect` using this `Rect`
    fn to_sdl_rect(&self) -> sdl2::rect::Rect {
        sdl2::rect::Rect::new(self.x as i32, self.y as i32, self.w as u32, self.h as u32)
    }
}

pub mod resource {
//! represent sdl2 textures and fonts as cheap structs that hold indexes for resource managers

    #[derive(Clone, Copy)]
    pub struct Texture {
        pub id:     usize,
        pub width:  u32,
        pub height: u32
    }
    #[derive(Clone, Copy)]
    pub struct Font {
        pub id : usize,
    }
}

#[derive(Clone, Copy)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Colour {
        Colour { r, g, b, a }
    }
    pub fn new_from_floats(r: f64, g: f64, b: f64, a: f64) -> Colour {
        Self::new(
            (r / 255.0) as u8,
            (g / 255.0) as u8,
            (b / 255.0) as u8,
            (a / 255.0) as u8,
        ) 
    }
    pub fn white() -> Colour {
        Self::new(255, 255, 255, 255)
    }

    pub fn to_sdl2_colour(&self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GameObject {
    texture: resource::Texture,
    rect: Rect,
    tex_rect: Rect,
    parallax: Vec2,
    colour: Colour
}

impl GameObject {
    pub fn new_from_tex(texture: resource::Texture) -> Self {
        let r = Rect::new(0.0, 0.0, texture.width as f64, texture.height as f64);
        Self {
            texture,
            rect: r,
            tex_rect : r,
            parallax: Vec2::new(1.0, 1.0),
            colour: Colour::white(),
        }
    }
    pub fn new(texture : resource::Texture, rect : Rect, tex_rect: Rect, parallax : Vec2, colour: Colour) -> Self {
        Self {
            texture,
            rect,
            tex_rect,
            parallax,
            colour,
        }
    }
}

/// holds a `Texture` and some `Rect`s for representing sprites
#[derive(Clone, Copy)]
pub struct TextureDraw {
    pub draw_rect : Rect,
    pub tex_rect : Rect,
    pub colour : Colour,
    pub tex  : resource::Texture,
}

impl TextureDraw {
    pub fn new(tex : resource::Texture, draw_rect : Rect, tex_rect: Rect, colour: Colour) -> Self {
        TextureDraw {
            draw_rect,
            tex_rect,
            colour,
            tex
        }
    }
}

/// stores textures that are referenced by a `resource::Texture` object
pub struct TextureManager<'a, T> {
    texture_creator : &'a TextureCreator<T>,
    loaded_texture_paths : HashMap<String,  usize>,
    textures     : Vec<Texture<'a>>,
}

impl<'a, T> TextureManager<'a, T> {
    pub fn new(tex_creator: &'a TextureCreator<T>) -> Self {

        TextureManager {
            texture_creator : tex_creator,
            loaded_texture_paths: HashMap::new(),
            textures : Vec::new(),
        }
    }
/// load a texture to memory and get a `resource::Texture` object that references it
    pub fn load(&mut self, path : &Path) -> Result<resource::Texture, String> {
        let path_as_string = path.to_string_lossy().to_string();
        let tex_index = match self.loaded_texture_paths.contains_key(&path_as_string) {
            true => self.loaded_texture_paths[&path_as_string],
            false => {
                self.textures.push(self.texture_creator.load_texture(path)?);
                self.loaded_texture_paths.insert(path_as_string, self.textures.len() - 1);

                println!("loaded: {}", path.to_str().unwrap());

                self.textures.len() - 1
            },
        };
        let last_tex = &self.textures[tex_index];
        Ok(
        resource::Texture {
            id: tex_index,
            width: last_tex.query().width,
            height: last_tex.query().height,
        })

    }
/// draw a `GameObject` to the canvas
    pub fn draw(&mut self, canvas : &mut Canvas<Window>, tex_draw: TextureDraw) -> Result<(), String> {
        self.textures[tex_draw.tex.id].set_color_mod(
            tex_draw.colour.r,
            tex_draw.colour.g,
            tex_draw.colour.b
        );
        self.textures[tex_draw.tex.id].set_alpha_mod(tex_draw.colour.a);
        canvas.copy(
            &self.textures[tex_draw.tex.id],
            tex_draw.tex_rect.to_sdl_rect(),
            tex_draw.draw_rect.to_sdl_rect()
        )
    }

    pub fn draw_rect(&self, canvas : &mut Canvas<Window>, rect : &geometry::Rect, colour : &geometry::Rect) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(colour.x as u8, colour.y as u8, colour.w as u8, colour.h as u8));
        canvas.fill_rect(rect.to_sdl_rect())?;
        Ok(())
    }
}

/// can be returned by `FontManager`, stores an sdl2 texture and a rect for drawing to a canvas
pub struct TextDraw<'a> {
    pub tex  : sdl2::render::Texture<'a>,
    pub rect : sdl2::rect::Rect,
}

const FONT_LOAD_SIZE : u16 = 128;

/// Stores 'sdl2::ttf::Font' and returns textures or draws them
pub struct FontManager<'a, T> {
    texture_creator : &'a TextureCreator<T>,
    ttf_context: &'a ttf::Sdl2TtfContext,
    loaded_font_paths : HashMap<String, usize>,
    pub fonts : Vec<ttf::Font<'a, 'static>>,
}

impl<'a, T> FontManager<'a, T> {
    pub fn new(ttf_context : &'a ttf::Sdl2TtfContext, texture_creator : &'a TextureCreator<T>) -> Result<Self, String> {
        Ok(FontManager {
            texture_creator,
            ttf_context,
            loaded_font_paths: HashMap::new(),
            fonts : Vec::new(),
        })
    }

    pub fn load_font(&mut self, path : &Path) -> Result<resource::Font, String>{
        let path_string = path.to_string_lossy().to_string();
        let font_index = match self.loaded_font_paths.contains_key(&path_string) {
            true => self.loaded_font_paths[&path_string],
            false => {
                self.fonts.push(
                    match self.ttf_context.load_font(path, FONT_LOAD_SIZE) {
                        Ok(s) => s,
                        Err(e) => { return Err(e.to_string()); }
                    }
                );
                self.loaded_font_paths.insert(path_string, self.fonts.len() - 1);
                self.fonts.len() - 1
            }
        };
        Ok(
            resource::Font {
            id: font_index,
        })
    }
    /// return a `TextDraw` that has a corrected `rect.width` based on the supplied height and the rendered font
    pub fn get_draw(&self, font: &resource::Font, text: &str, height : u32, colour : Color) -> Result<TextDraw, String> {
        self.get_draw_at_vec2(font, text, height, Vec2::new(0.0, 0.0), colour)
    }

    pub fn get_draw_at_vec2(&self, font: &resource::Font, text: &str, height : u32, pos: Vec2, colour: Color) -> Result<TextDraw, String> {
        if text.len() == 0 { Err("text length should be greater than 0")?; }
        let surface = match self.fonts[font.id]
            .render(text)
            .blended(colour) {
                Ok(s) => s,
                Err(e) => return Err(e.to_string()),
        };
        let tex = match self.texture_creator.create_texture_from_surface(&surface) {
            Ok(t) => t,
            Err(e) => { return Err(e.to_string()); },
        };
        let ratio = tex.query().height as f64 / tex.query().width as f64;
        Ok(
        TextDraw {
            tex,
            rect:
             sdl2::rect::Rect::new(
                pos.x as i32,
                pos.y as i32,
                (height as f64 / ratio) as u32,
                height
             ),
        })
    }

    /// draws the supplied text to the canvas in the supplied font at the given height and position
    pub fn draw(&self, canvas : &mut Canvas<Window>, font : &resource::Font, text: &str, height : u32, pos : Vec2, colour : Color) -> Result<(), String> {
        if text.len() == 0 { return Ok(()); }
        let mut tex_draw = self.get_draw(font, text, height, colour)?;
        tex_draw.rect.x = pos.x as i32;
        tex_draw.rect.y = pos.y as i32;
        canvas.copy(&tex_draw.tex, None, tex_draw.rect)
    }
}
