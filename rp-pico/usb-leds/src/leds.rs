use crate::colors::{hsv_interp, Pixel, Pixel16, PixelHsv};
use crate::sprites::RandomSprites;

#[derive(PartialEq)]
pub enum LedType {
    Apa102,
    Ws2801,
    Analog,
}

pub enum BufferType {
    Addressable([u8; 4612]),
    Analog([u16; 3]),
}

pub struct Leds {
    led_type: LedType,
    len: usize,
    buf_end: usize,
    //~ buffer: [u8; 4612],
    buffer: BufferType,
}

impl Leds {
    pub fn new(mut led_count: usize, led_type: LedType) -> Self {
        // LED count is limited to 1024
        assert!(led_count <= 1024);
        let buf_end = match led_type {
            LedType::Apa102 => 4 + (led_count * 4) + ((led_count + 1) / 2),
            LedType::Ws2801 => led_count * 3,
            LedType::Analog => 3,
        };
        if led_type == LedType::Analog { led_count = 1; }
        match led_type {
            LedType::Apa102 | LedType::Ws2801 => {
                Self {
                    led_type: led_type,
                    len: led_count,
                    buf_end: buf_end,
                    buffer: BufferType::Addressable([0; 4612]),
                }
            }
            LedType::Analog => {
                Self {
                    led_type: led_type,
                    len: led_count,
                    buf_end: buf_end,
                    buffer: BufferType::Analog([0; 3]),
                }
            }
        }
        
        //~ Self {
            //~ led_type: led_type,
            //~ len: led_count,
            //~ buf_end: buf_end,
            //~ buffer: [0; 4612],
        //~ }
    }

    pub fn all_off(&mut self) {
        match self.led_type {
            LedType::Apa102 => {
                //~ for (i, v) in self.buffer.chunks_mut(4).skip(1).enumerate() {
                    //~ if i >= self.len { break; }
                    //~ v[0] = 255;
                    //~ v[1] = 0;
                    //~ v[2] = 0;
                    //~ v[3] = 0;
                //~ }
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(4).skip(1).enumerate() {
                            if i >= self.len { break; }
                            v[0] = 255;
                            v[1] = 0;
                            v[2] = 0;
                            v[3] = 0;
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {
                //~ for (i, v) in self.buffer.iter_mut().enumerate() {
                    //~ *v = 0;
                //~ }
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for v in b.iter_mut() {
                            *v = 0;
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {
                //~ for (i, v) in self.buffer.iter_mut().enumerate() {
                    //~ *v = 0;
                //~ }
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        for v in b.iter_mut() {
                            *v = 0;
                        }
                    }
                }
            }
        }
    }
    
    pub fn led_type(&self) -> &LedType {
        &self.led_type
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn buf_end(&self) -> usize {
        self.buf_end
    }
    
    pub fn buffer(&self) -> &BufferType {
        &self.buffer
    }
    
    pub fn set_led(&mut self, rgb: Pixel, index: usize) {
        match self.led_type {
            LedType::Apa102 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        match b.chunks_mut(4).skip(1).nth(index) {
                            Some(v) => {
                                set_array_apa102(v, rgb);
                            },
                            None => {},
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        match b.chunks_mut(3).nth(index) {
                            Some(v) => {
                                set_array_ws2801(v, rgb);
                            },
                            None => {},
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        match b.chunks_mut(3).nth(0) {
                            Some(v) => {
                                set_array_analog8(v, rgb);
                            },
                            None => {},
                        }
                    }
                }
            }
        }
    }

    pub fn fill_gradient(&mut self, start: &PixelHsv, end: &PixelHsv) {
        match self.led_type {
            LedType::Apa102 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(4).skip(1).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let rgb = hsv_interp(&start, &end, pos).to_rgb();
                            set_array_apa102(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(3).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let rgb = hsv_interp(&start, &end, pos).to_rgb();
                            set_array_ws2801(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        match b.chunks_mut(3).nth(0) {
                            Some(v) => {
                                set_array_analog16(v, start.to_rgb16());
                            },
                            None => {},
                        }
                    }
                }
            }
        }
    }

    pub fn fill_gradient_dual(&mut self, start: &PixelHsv, end: &PixelHsv) {
        match self.led_type {
            LedType::Apa102 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(4).skip(1).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let mut pos_bipolar = pos * 2.0 - 1.0;
                            if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                            let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                            set_array_apa102(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {                
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(3).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let mut pos_bipolar = pos * 2.0 - 1.0;
                            if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                            let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                            set_array_ws2801(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {                
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        match b.chunks_mut(3).nth(0) {
                            Some(v) => {
                                set_array_analog16(v, start.to_rgb16());
                            },
                            None => {},
                        }
                    }
                }
            }
        }
    }

    pub fn fill_triangle(&mut self, start: &PixelHsv, end: &PixelHsv, phase: f32) {
        match self.led_type {
            LedType::Apa102 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(4).skip(1).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let mut pos_triangle = ((pos + phase) % 1.0) * 2.0 - 1.0;
                            if pos_triangle < 0.0 { pos_triangle = pos_triangle * -1.0 }
                            let rgb = hsv_interp(&end, &start, pos_triangle).to_rgb();
                            set_array_apa102(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {                
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        for (i, v) in b.chunks_mut(3).enumerate() {
                            if i >= self.len { break; }
                            let pos = position(i, self.len);
                            let mut pos_triangle = ((pos + phase) % 1.0) * 2.0 - 1.0;
                            if pos_triangle < 0.0 { pos_triangle = pos_triangle * -1.0; }
                            let rgb = hsv_interp(&end, &start, pos_triangle).to_rgb();
                            set_array_ws2801(v, rgb);
                        }
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        match b.chunks_mut(3).nth(0) {
                            Some(v) => {
                                let mut triangle = phase * 2.0 - 1.0;
                                if triangle < 0.0 { triangle = triangle * -1.0; }
                                let rgb = hsv_interp(&start, &end, triangle).to_rgb16();
                                set_array_analog16(v, rgb);
                            },
                            None => {},
                        }
                    }
                }
            }
        }
    }
    
    pub fn fill_random(&mut self, start: &PixelHsv, end: &PixelHsv, sprites: &RandomSprites) {
        match self.led_type {
            LedType::Apa102 => {
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        b.chunks_mut(4).skip(1).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                            let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                            set_array_apa102(led, rgb);
                        });
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Ws2801 => {                
                match self.buffer {
                    BufferType::Addressable(ref mut b) => {
                        b.chunks_mut(3).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                            let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                            set_array_ws2801(led, rgb);
                        });
                    }
                    BufferType::Analog(_) => {}
                }
            }
            LedType::Analog => {                
                match self.buffer {
                    BufferType::Addressable(_) => {}
                    BufferType::Analog(ref mut b) => {
                        b.chunks_mut(3).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                            let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb16();
                            set_array_analog16(led, rgb);
                        });
                    }
                }
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

fn set_array_analog8(led: &mut [u16], rgb: Pixel) {
    led[0] = (rgb.get_r() as u16) << 8;
    led[1] = (rgb.get_g() as u16) << 8;
    led[2] = (rgb.get_b() as u16) << 8;
}

fn set_array_analog16(led: &mut [u16], rgb: Pixel16) {
    led[0] = rgb.get_r();
    led[1] = rgb.get_g();
    led[2] = rgb.get_b();
}
