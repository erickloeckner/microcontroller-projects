use crate::colors::{hsv_interp, Pixel, PixelHsv};
use crate::sprites::RandomSprites;
use crate::NUM_LEDS;

pub enum LedType {
    Apa102,
    Ws2801,
}

pub struct Leds<const L: usize, const B: usize> {
    led_type: LedType,
    len: usize,
    buffer: [u8; B],
}

impl<const L: usize, const B: usize> Leds<L, B> {
    pub fn new(led_type: LedType) -> Self {
        Self {
            led_type: led_type,
            len: L,
            buffer: [0; B],
        }
    }

    pub fn all_off(&mut self) {
        match self.led_type {
            LedType::Apa102 => {
                let len = self.len;
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < len {
                        v[0] = 255;
                        v[1] = 0;
                        v[2] = 0;
                        v[3] = 0;
                    }
                });
            }
            LedType::Ws2801 => {
                self.buffer.iter_mut().for_each(|v| *v = 0 );
            }
        }
    }
    
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
    
    pub fn set_led(&mut self, pixel: Pixel, index: usize) {
        if index < self.len {
            match self.led_type {
                LedType::Apa102 => {
                    match self.buffer.chunks_mut(4).skip(1).nth(index) {
                        Some(v) => {
                            v[0] = 255;
                            v[1] = pixel.get_b();
                            v[2] = pixel.get_g();
                            v[3] = pixel.get_r();
                        },
                        None => {},
                    }
                }
                LedType::Ws2801 => {
                    match self.buffer.chunks_mut(3).nth(index) {
                        Some(v) => {
                            v[0] = pixel.get_r();
                            v[1] = pixel.get_g();
                            v[2] = pixel.get_b();
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
                let len = self.len;
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < len {
                        let pos = (i as f32) / ((len - 1) as f32);
                        let rgb = hsv_interp(&start, &end, pos).to_rgb();
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let rgb = hsv_interp(&start, &end, pos).to_rgb();
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }

    pub fn fill_gradient_dual(&mut self, start: &PixelHsv, end: &PixelHsv) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < self.len {
                        let pos = (i as f32) / ((self.len - 1) as f32);
                        let mut pos_bipolar = pos * 2.0 - 1.0;
                        if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                        let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }
    
    pub fn fill_random(&mut self, start: &PixelHsv, end: &PixelHsv, sprites: &RandomSprites<{ NUM_LEDS }>) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    led[0] = 255;
                    led[1] = rgb.get_b();
                    led[2] = rgb.get_g();
                    led[3] = rgb.get_r();
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    led[0] = rgb.get_r();
                    led[1] = rgb.get_g();
                    led[2] = rgb.get_b();
                });
            }
        }
    }
}
