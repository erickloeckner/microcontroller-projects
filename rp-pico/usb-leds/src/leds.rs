use crate::colors::{hsv_interp, Pixel, PixelHsv};
use crate::sprites::RandomSprites;

pub enum LedType {
    Apa102,
    Ws2801,
}

pub struct Leds {
    led_type: LedType,
    len: usize,
    buf_end: usize,
    buffer: [u8; 4612],
}

impl Leds {
    pub fn new(led_count: usize, led_type: LedType) -> Self {
        // LED count is limited to 1024
        assert!(led_count <= 1024);
        let buf_end = match led_type {
            LedType::Apa102 => 4 + (led_count * 4) + ((led_count + 1) / 2),
            LedType::Ws2801 => led_count * 3,
        };
        Self {
            led_type: led_type,
            len: led_count,
            buf_end: buf_end,
            buffer: [0; 4612],
        }
    }

    pub fn all_off(&mut self) {
        match self.led_type {
            LedType::Apa102 => {
                for (i, v) in self.buffer.chunks_mut(4).skip(1).enumerate() {
                    if i >= self.len { break; }
                    v[0] = 255;
                    v[1] = 0;
                    v[2] = 0;
                    v[3] = 0;
                }
            }
            LedType::Ws2801 => {
                for (i, v) in self.buffer.iter_mut().enumerate() {
                    if i >= self.len { break; }
                    *v = 0;
                }
            }
        }
    }
    
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer[0..self.buf_end]
    }
    
    pub fn set_led(&mut self, rgb: Pixel, index: usize) {
        if index < self.len {
            match self.led_type {
                LedType::Apa102 => {
                    match self.buffer.chunks_mut(4).skip(1).nth(index) {
                        Some(v) => {
                            set_array_apa102(v, rgb);
                        },
                        None => {},
                    }
                }
                LedType::Ws2801 => {
                    match self.buffer.chunks_mut(3).nth(index) {
                        Some(v) => {
                            set_array_ws2801(v, rgb);
                        },
                        None => {},
                    }
                }
            }
        }
    }

    pub fn fill_gradient(&mut self, start: &PixelHsv, end: &PixelHsv) {
        match self.led_type {
            LedType::Apa102 => {
                for (i, v) in self.buffer.chunks_mut(4).skip(1).enumerate() {
                    if i >= self.len { break; }
                    let pos = position(i, self.len);
                    let rgb = hsv_interp(&start, &end, pos).to_rgb();
                    set_array_apa102(v, rgb);
                }
            }
            LedType::Ws2801 => {
                for (i, v) in self.buffer.chunks_mut(3).enumerate() {
                    if i >= self.len { break; }
                    let pos = position(i, self.len);
                    let rgb = hsv_interp(&start, &end, pos).to_rgb();
                    set_array_ws2801(v, rgb);
                }
            }
        }
    }

    pub fn fill_gradient_dual(&mut self, start: &PixelHsv, end: &PixelHsv) {
        match self.led_type {
            LedType::Apa102 => {
                for (i, v) in self.buffer.chunks_mut(4).skip(1).enumerate() {
                    if i >= self.len { break; }
                    let pos = position(i, self.len);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                    set_array_apa102(v, rgb);
                }
            }
            LedType::Ws2801 => {                
                for (i, v) in self.buffer.chunks_mut(3).enumerate() {
                    if i >= self.len { break; }
                    let pos = position(i, self.len);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                    set_array_ws2801(v, rgb);
                }
            }
        }
    }

    pub fn fill_triangle(&mut self, start: &PixelHsv, end: &PixelHsv, phase: f32) {
        match self.led_type {
            LedType::Apa102 => {
                for (i, v) in self.buffer.chunks_mut(4).skip(1).enumerate() {
                    if i >= self.len { break; }
                    let pos = position(i, self.len);
                    let mut pos_triangle = (((pos + phase) % 1.0) * 2.0 - 1.0);
                    if pos_triangle < 0.0 { pos_triangle = pos_triangle * -1.0 }
                    let rgb = hsv_interp(&end, &start, pos_triangle).to_rgb();
                    set_array_apa102(v, rgb);
                }
            }
            LedType::Ws2801 => {                
                for (i, v) in self.buffer.chunks_mut(3).enumerate() {
                    if i >= self.len { break; }
                    //let pos = (i as f32) / ((self.len - 1) as f32);
                    let pos = position(i, self.len);
                    let mut pos_triangle = (((pos + phase) % 1.0) * 2.0 - 1.0);
                    if pos_triangle < 0.0 { pos_triangle = pos_triangle * -1.0; }
                    let rgb = hsv_interp(&end, &start, pos_triangle).to_rgb();
                    set_array_ws2801(v, rgb);
                }
            }
        }
    }
    
    pub fn fill_random(&mut self, start: &PixelHsv, end: &PixelHsv, sprites: &RandomSprites) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    /*
                    led[0] = 255;
                    led[1] = rgb.get_b();
                    led[2] = rgb.get_g();
                    led[3] = rgb.get_r();
                    */
                    set_array_apa102(led, rgb);
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    /*
                    led[0] = rgb.get_r();
                    led[1] = rgb.get_g();
                    led[2] = rgb.get_b();
                    */
                    set_array_ws2801(led, rgb);
                });
            }
        }
    }
}

fn position(index: usize, len: usize) -> f32 {
    (index as f32) / ((len - 1) as f32)
}

fn set_array_apa102(led: &mut [u8], rgb: Pixel) {
    led[0] = 255;
    led[1] = rgb.get_b();
    led[2] = rgb.get_g();
    led[3] = rgb.get_r();
}

fn set_array_ws2801(led: &mut [u8], rgb: Pixel) {
    led[0] = rgb.get_r();
    led[1] = rgb.get_g();
    led[2] = rgb.get_b();
}
