//! take sdl2 events and update a struct of bools for required controls

use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;

/// Holds mouse input info
#[derive(Copy, Clone)]
pub struct Mouse {
    pub x : i32,
    pub y : i32,
    pub left_click : bool,
    pub right_click : bool,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            x: 0,
            y: 0,
            left_click : false,
            right_click : false,
        }
    }
}


/// Holds character typed that frame, and the state of some useful buttons for typing
#[derive(Copy, Clone)]
pub struct Input {
    pub up        : bool,
    pub down      : bool,
    pub left      : bool,
    pub right     : bool,
    pub a         : bool,
    pub b         : bool,
    pub restart: bool,
    pub debug_1   : bool,
    pub debug_2   : bool,
    pub debug_3   : bool,
    pub mouse     : Mouse,
}

impl Input {

    pub fn new() -> Self {
        Input {
            up        : false,
            down      : false,
            left      : false,
            right     : false,
            a         : false,
            b         : false,
            restart : false,
            mouse     : Mouse::new(),
            debug_1: false,
            debug_2: false,
            debug_3: false,
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        if event.is_keyboard() {
            self.handle_keyboard(event);
        } else if event.is_mouse() {
            self.handle_mouse(event);
        }
    }

    fn handle_keyboard(&mut self, event : &Event) {
        let mut key_down = false;
        let key = match event {
            Event::KeyDown {
                scancode: k,
                ..
            } => {
                key_down = true;
                k
            },
            Event::KeyUp {
                scancode: k,
                ..
            } => k,
            _ => &None
        };
        match key {
            Some(k) => {
                match k {
                    Scancode::Up | Scancode::W => self.up    = key_down,
                    Scancode::Left | Scancode::A => self.left  = key_down,
                    Scancode::Down | Scancode::S => self.down  = key_down,
                    Scancode::Right | Scancode::D => self.right = key_down,
                    Scancode::Z | Scancode::Comma => self.a = key_down,
                    Scancode::X | Scancode::Period => self.b = key_down,
                    Scancode::R => self.restart = key_down,
                    Scancode::F1 => self.debug_1 = key_down,
                    Scancode::F2 => self.debug_2 = key_down,
                    Scancode::F3 => self.debug_3 = key_down,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, event : &Event) {
        let mut btn_down = false;
        let btn = match event {
            Event::MouseMotion { x, y, .. } => {
                self.mouse.x = *x;
                self.mouse.y = *y;
                None
            },
            Event::MouseButtonDown { mouse_btn, ..} => {
                btn_down = true;
                Some(mouse_btn)
            },
            Event::MouseButtonUp { mouse_btn, .. } => {
                btn_down = false;
                Some(mouse_btn)
            }
            _ => None,
        };
        match btn {
            Some(btn) => match btn {
                MouseButton::Left => self.mouse.left_click = btn_down,
                MouseButton::Right => self.mouse.right_click = btn_down,
                _ => (),
            }
            None => (),
        }
    }
}
