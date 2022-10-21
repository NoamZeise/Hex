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
pub struct Typing {
    pub up        : bool,
    pub down      : bool,
    pub left      : bool,
    pub right     : bool,
    pub a         : bool,
    pub b         : bool,
    pub mouse     : Mouse,
    character     : Option<char>,
}

impl Typing {

    pub fn new() -> Self {
        Typing {
            up        : false,
            down      : false,
            left      : false,
            right     : false,
            a         : false,
            b         : false,
            mouse     : Mouse::new(),
            character : None
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        if event.is_keyboard() {
            self.handle_keyboard(event);
        } else if event.is_text() {
            self.handle_text(event);
        } else if event.is_mouse() {
            self.handle_mouse(event);
        }
    }

    pub fn get_character(&mut self) -> Option<char> {
        match self.character {
            Some(c) => {
                self.character = None;
                Some(c)
            },
            None => None
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
                    Scancode::Up => self.up    = key_down,
                    Scancode::Left => self.left  = key_down,
                    Scancode::Down => self.down  = key_down,
                    Scancode::Right => self.right = key_down,
                    Scancode::Z => self.a = key_down,
                    Scancode::X => self.b = key_down,
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_text(&mut self, event : &Event) {
        self.character = match event {
            Event::TextInput { text : t, ..} => {
                if t.len() > 0 {
                    Some(t.chars().nth(0).unwrap())
                } else {
                    None
                }
                },
            _ => None,
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
