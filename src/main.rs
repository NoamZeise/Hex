
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::image;
use sdl2::video::Window;
use sdl2::render::Canvas;

use geometry::Vec2;
use gudevJam12::{
    TextureManager,
    FontManager,
    map,
    camera::Camera,
    input::Input,
    hex::HexGrid,
};

use std::time::Instant;
use std::path::Path;

const TARGET_WIDTH : f64 = 240.0;
const TARGET_HEIGHT : f64 = 160.0;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = image::init(image::InitFlag::PNG);

    let mut cam = Camera::new(
        geometry::Rect::new(0.0, 0.0, TARGET_WIDTH, TARGET_HEIGHT),
        geometry::Vec2::new(TARGET_WIDTH * 4.0, TARGET_HEIGHT * 4.0)
    );
    
    let window = video_subsystem
        .window(
            "Deeper and Deeper",
            cam.get_window_size().x as u32,
            cam.get_window_size().y as u32
        )
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture_manager = TextureManager::new(&texture_creator);
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let mut font_manager = FontManager::new(&ttf_context, &texture_creator)?;

    let mono_font = font_manager.load_font(Path::new("textures/VT323-Regular.ttf"))?;

    let test = gudevJam12::GameObject::new_from_tex(texture_manager.load(Path::new("textures/bg.png"))?);

    let mut hex_grid = HexGrid::new(&mut texture_manager)?;
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Mul);

    let mut palette = Color::RGBA(0, 0, 0, 0);

    let mut highscore = 0;
    let mut new_hs = false;

    let mut event_pump = sdl_context.event_pump()?;
    let mut input = Input::new();
    let mut p_inp = input;
    let mut prev_frame : f64 = 0.0;
    'running: loop {
        let start_time = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {  keycode: Some(Keycode::Escape), ..} => break 'running,
                _ => { }
            }
            input.handle_event(&event);
            handle_event(&event, &mut canvas, &mut cam)?;
        }
        
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        
        //map.draw(&mut cam);
        cam.add_cam_space(&test);
        
        hex_grid.draw(&mut cam);
        
        for d in cam.drain_draws() {
            texture_manager.draw(&mut canvas, d)?;
        }

        let cam_x = cam.get_window_size().x / cam.get_view_size().x;
        font_manager.draw(&mut canvas, &mono_font, &format!("score: {}", hex_grid.score()), (7.0*cam_x) as u32, Vec2::new(15.0*cam_x, 10.0*cam_x), Color::RGB(178, 178, 178))?;
        font_manager.draw(&mut canvas, &mono_font, &format!("highscore: {}", hex_grid.score()), (7.0*cam_x) as u32, Vec2::new(4.0*cam_x, 18.0*cam_x), Color::RGB(178, 178, 178))?;

        canvas.set_draw_color(Color::RGB(32, 31, 46));
        let width = 20;
        let height = (cam.get_window_size().y / hex_grid.spawn_ratio())as u32;
        canvas.fill_rect(sdl2::rect::Rect::new(cam.get_window_size().x as i32 - width, (cam.get_window_size().y - height as f64) as i32,
                                               width as u32, height))?;
                                               
        
        canvas.set_draw_color(palette);

        if hex_grid.lost() {
            
            canvas.set_draw_color(Color::RGBA(10, 10, 10, 200));
        }
        
        canvas.fill_rect(sdl2::rect::Rect::new(0, 0, cam.get_window_size().x as u32, cam.get_window_size().y as u32))?;

        if hex_grid.lost() {
            font_manager.draw(&mut canvas, &mono_font, "GAME OVER",
                              (40.0*cam_x) as u32,
                              Vec2::new(45.0*cam_x, 30.0*cam_x),
                              Color::RGB(200, 200, 200))?;

            font_manager.draw(&mut canvas, &mono_font, &format!("FINAL SCORE: {}", hex_grid.score()),
                              (20.0*cam_x) as u32,
                              Vec2::new(57.0*cam_x, 67.0*cam_x),
                              Color::RGB(200, 200, 200))?;

            if new_hs {
                 font_manager.draw(&mut canvas, &mono_font, "NEW HIGH SCORE!",
                              (15.0*cam_x) as u32,
                              Vec2::new(78.0*cam_x, 110.0*cam_x),
                              Color::RGB(200, 200, 200))?;

            }

             font_manager.draw(&mut canvas, &mono_font, "Z TO RETRY",
                              (10.0*cam_x) as u32,
                              Vec2::new(100.0*cam_x, 130.0*cam_x),
                              Color::RGB(200, 200, 200))?;
            
        }
        
        canvas.present(); 

        if hex_grid.score() < 30 {
            palette = Color::RGBA(255, 255, 255, 255);
        } else if hex_grid.score() < 70 {
            palette = Color::RGBA(150, 100, 220, 160);
        } else if hex_grid.score() < 120 {
            palette = Color::RGBA(255, 100, 0, 160);
        } else if hex_grid.score() < 160 {
            palette = Color::RGBA(155, 255, 100, 170);
        } else if hex_grid.score() < 210 {
            palette = Color::RGBA(100, 200, 255, 200);
        } else if hex_grid.score() < 260 {
            palette = Color::RGBA(255, 255, 100, 160);
        } else if hex_grid.score() < 300 {
            palette = Color::RGBA(200, 170, 255, 100);
        } else if hex_grid.score() < 300 {
            palette = Color::RGBA(255, 90, 100, 150);
        } else if hex_grid.score() < 350 {
            palette = Color::RGBA(200, 90, 200, 200)
        } else if hex_grid.score() < 400 {
            palette = Color::RGBA(255, 90, 60, 230);
        } else if hex_grid.score() < 450 {
            palette = Color::RGBA(255, 90, 60, 240);
        } else if hex_grid.score() < 500 {
            palette = Color::RGBA(255, 40, 40, 250);
        }
        
        if hex_grid.score() > highscore {
            highscore = hex_grid.score();
            new_hs = true;
        }
        
        if hex_grid.lost() {
            if input.a && !p_inp.a{
                hex_grid.reset();
                new_hs = false;
            }
        } else {
            hex_grid.update(&prev_frame, &input);
        }
              
        
        prev_frame = start_time.elapsed().as_secs_f64();
        p_inp = input;
        //println!("prev frame: {} fps", 1.0/prev_frame);
    }

    Ok(())
}


fn handle_event(event: &Event, canvas: &mut Canvas<Window>, cam: &mut Camera) -> Result<(), String> {
    match event {
        Event::KeyDown {
            keycode: Some(Keycode::Equals),
            ..
        } => {
            let mut cs = cam.get_window_size();
            if cs.x < cam.get_view_size().x {
                cs.x *= 2.0;
                cs.y *= 2.0;
            } else {
                cs.x += cam.get_view_size().x;
                cs.y += cam.get_view_size().y;
            }
            set_win_size(canvas, cam, cs)?;
        },
        Event::KeyDown {
            keycode: Some(Keycode::Minus),
            ..
        } => {
            let mut cs = cam.get_window_size();
            if cs.x <= cam.get_view_size().x {
                cs.x /= 2.0;
                cs.y /= 2.0;
            } else {
                cs.x -= cam.get_view_size().x;
                cs.y -= cam.get_view_size().y;
            }
            set_win_size(canvas, cam, cs)?;
        },
        _ => {}
    }
    Ok(())
}

fn set_win_size(canvas: &mut Canvas<Window>, cam: &mut Camera, cs: Vec2) -> Result<(), String> {
    match canvas.window_mut().set_size(cs.x as u32, cs.y as u32) {
        Err(_) => { return Err(String::from("failed to resize window"));},
        _ => ()
    }
    cam.set_window_size(cs);
    canvas.window_mut().set_position(
        sdl2::video::WindowPos::Centered,
        sdl2::video::WindowPos::Centered
    );
    Ok(())
}
