use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct NativeGraphicsContext {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    keys_pressed: HashMap<Key, bool>,
    mouse_pos: (f32, f32),
    mouse_buttons: [bool; 3],
}

impl NativeGraphicsContext {
    pub fn new(width: usize, height: usize, title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut window = Window::new(
            title,
            width,
            height,
            WindowOptions {
                resize: true,
                ..WindowOptions::default()
            },
        )?;

        window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // ~60 FPS

        Ok(Self {
            window,
            buffer: vec![0; width * height],
            width,
            height,
            keys_pressed: HashMap::new(),
            mouse_pos: (0.0, 0.0),
            mouse_buttons: [false; 3],
        })
    }

    pub fn update_input(&mut self) {
        // Обновляем состояние клавиш
        for key in [
            Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J,
            Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T,
            Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
            Key::Key0, Key::Key1, Key::Key2, Key::Key3, Key::Key4,
            Key::Key5, Key::Key6, Key::Key7, Key::Key8, Key::Key9,
            Key::Up, Key::Down, Key::Left, Key::Right,
            Key::Space, Key::Enter, Key::Escape,
        ] {
            self.keys_pressed.insert(key, self.window.is_key_down(key));
        }

        // Обновляем состояние мыши
        if let Some((x, y)) = self.window.get_mouse_pos(MouseMode::Clamp) {
            self.mouse_pos = (x, y);
        }

        self.mouse_buttons[0] = self.window.get_mouse_down(MouseButton::Left);
        self.mouse_buttons[1] = self.window.get_mouse_down(MouseButton::Right);
        self.mouse_buttons[2] = self.window.get_mouse_down(MouseButton::Middle);
    }

    pub fn clear_screen(&mut self, r: u8, g: u8, b: u8) {
        let color = Self::rgb_to_u32(r, g, b);
        for pixel in self.buffer.iter_mut() {
            *pixel = color;
        }
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, r: u8, g: u8, b: u8) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let index = (y as usize) * self.width + (x as usize);
            self.buffer[index] = Self::rgb_to_u32(r, g, b);
        }
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, r: u8, g: u8, b: u8) {
        let color = Self::rgb_to_u32(r, g, b);
        for py in y..(y + height) {
            for px in x..(x + width) {
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let index = (py as usize) * self.width + (px as usize);
                    self.buffer[index] = color;
                }
            }
        }
    }

    pub fn draw_circle(&mut self, center_x: i32, center_y: i32, radius: i32, r: u8, g: u8, b: u8) {
        let color = Self::rgb_to_u32(r, g, b);
        let radius_sq = radius * radius;

        for y in (center_y - radius)..=(center_y + radius) {
            for x in (center_x - radius)..=(center_x + radius) {
                let dx = x - center_x;
                let dy = y - center_y;
                if dx * dx + dy * dy <= radius_sq {
                    if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                        let index = (y as usize) * self.width + (x as usize);
                        self.buffer[index] = color;
                    }
                }
            }
        }
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, r: u8, g: u8, b: u8) {
        let color = Self::rgb_to_u32(r, g, b);
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                let index = (y as usize) * self.width + (x as usize);
                self.buffer[index] = color;
            }

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn render(&mut self) -> bool {
        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)
            .is_ok()
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        *self.keys_pressed.get(&key).unwrap_or(&false)
    }

    pub fn get_mouse_pos(&self) -> (i32, i32) {
        (self.mouse_pos.0 as i32, self.mouse_pos.1 as i32)
    }

    pub fn is_mouse_button_pressed(&self, button: usize) -> bool {
        if button < self.mouse_buttons.len() {
            self.mouse_buttons[button]
        } else {
            false
        }
    }

    pub fn window_is_open(&self) -> bool {
        self.window.is_open()
    }

    fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }

    pub fn get_time(&self) -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }
}

// API совместимое с WASM версией
pub struct NativeGraphicsAPI {
    pub context: NativeGraphicsContext,
}

impl NativeGraphicsAPI {
    pub fn new(width: usize, height: usize, title: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            context: NativeGraphicsContext::new(width, height, title)?,
        })
    }

    pub fn init_graphics(&mut self, width: i32, height: i32, _title_ptr: i32) {
        // Размеры уже установлены при создании
        println!("Graphics initialized: {}x{}", width, height);
    }

    pub fn clear_screen(&mut self, r: i32, g: i32, b: i32) {
        self.context.clear_screen(
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        );
    }

    pub fn draw_pixel(&mut self, x: i32, y: i32, r: i32, g: i32, b: i32) {
        self.context.draw_pixel(
            x, y,
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        );
    }

    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, r: i32, g: i32, b: i32) {
        self.context.draw_rect(
            x, y, width, height,
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        );
    }

    pub fn draw_circle(&mut self, center_x: i32, center_y: i32, radius: i32, r: i32, g: i32, b: i32) {
        self.context.draw_circle(
            center_x, center_y, radius,
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        );
    }

    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, r: i32, g: i32, b: i32) {
        self.context.draw_line(
            x1, y1, x2, y2,
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        );
    }

    pub fn render(&mut self) -> bool {
        self.context.update_input();
        self.context.render()
    }

    pub fn get_time(&self) -> f64 {
        self.context.get_time()
    }

    pub fn is_key_pressed(&self, key_code: i32) -> bool {
        match key_code {
            32 => self.context.is_key_pressed(Key::Space), // Space
            37 => self.context.is_key_pressed(Key::Left),  // Left arrow
            38 => self.context.is_key_pressed(Key::Up),    // Up arrow
            39 => self.context.is_key_pressed(Key::Right), // Right arrow
            40 => self.context.is_key_pressed(Key::Down),  // Down arrow
            65 => self.context.is_key_pressed(Key::A),     // A
            87 => self.context.is_key_pressed(Key::W),     // W
            83 => self.context.is_key_pressed(Key::S),     // S
            68 => self.context.is_key_pressed(Key::D),     // D
            _ => false,
        }
    }

    pub fn get_mouse_pos(&self) -> (i32, i32) {
        self.context.get_mouse_pos()
    }

    pub fn is_mouse_button_pressed(&self, button: i32) -> bool {
        self.context.is_mouse_button_pressed(button as usize)
    }

    pub fn window_is_open(&self) -> bool {
        self.context.window_is_open()
    }
}