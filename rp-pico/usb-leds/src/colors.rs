#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl Pixel {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r,
            g: g,
            b: b,
        }
    }

    pub fn set_rgb(&mut self, rgb: &Pixel) {
        self.r = rgb.r;
        self.g = rgb.g;
        self.b = rgb.b;
    }

    pub fn set_r(&mut self, r: u8) {
        self.r = r;
    }

    pub fn get_r(&self) -> u8 {
        self.r
    }

    pub fn set_g(&mut self, g: u8) {
        self.g = g;
    }

    pub fn get_g(&self) -> u8 {
        self.g
    }

    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pixel16 {
    r: u16,
    g: u16,
    b: u16,
}

impl Pixel16 {
    pub fn new(r: u16, g: u16, b: u16) -> Self {
        Self {
            r: r,
            g: g,
            b: b,
        }
    }

    pub fn set_rgb(&mut self, rgb: &Pixel16) {
        self.r = rgb.r;
        self.g = rgb.g;
        self.b = rgb.b;
    }

    pub fn set_r(&mut self, r: u16) {
        self.r = r;
    }

    pub fn get_r(&self) -> u16 {
        self.r
    }

    pub fn set_g(&mut self, g: u16) {
        self.g = g;
    }

    pub fn get_g(&self) -> u16 {
        self.g
    }

    pub fn set_b(&mut self, b: u16) {
        self.b = b;
    }

    pub fn get_b(&self) -> u16 {
        self.b
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelHsv {
    h: f32,
    s: f32,
    v: f32,
}

impl PixelHsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { 
            h: h.max(0.0).min(1.0), 
            s: s.max(0.0).min(1.0), 
            v: v.max(0.0).min(1.0),
        }
    }

    pub fn set_hsv(&mut self, hsv: &PixelHsv) {
        self.h = hsv.h.max(0.0).min(1.0);
        self.s = hsv.s.max(0.0).min(1.0);
        self.v = hsv.v.max(0.0).min(1.0);
    }

    pub fn set_h(&mut self, h: f32) {
        self.h = h.max(0.0).min(1.0);
    }

    pub fn get_h(&self) -> f32 {
        self.h
    }

    pub fn set_s(&mut self, s: f32) {
        self.s = s.max(0.0).min(1.0);
    }

    pub fn get_s(&self) -> f32 {
        self.s
    }

    pub fn set_v(&mut self, v: f32) {
        self.v = v.max(0.0).min(1.0);
    }

    pub fn get_v(&self) -> f32 {
        self.v
    }

    pub fn to_rgb(&self) -> Pixel {
        let mut out = Pixel { r: 0, g: 0, b: 0 };
        let h_decimal = (self.h * 6.0) - (((self.h * 6.0) as u8) as f32);
        match (self.h * 6.0 % 6.0) as u8 {
            0 => {
                out.r = (self.v * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s)) * 255.0) as u8;
            }
            1 => {
                out.r = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
                out.g = (self.v * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s)) * 255.0) as u8;
            }
            2 => {
                out.r = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.g = (self.v * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
            }
            3 => {
                out.r = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
                out.b = (self.v * 255.0) as u8;
            }
            4 => {
                out.r = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.b = (self.v * 255.0) as u8;
            }
            5 => {
                out.r = (self.v * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
            }
            _ => (),
        }
        out
    }
    
    pub fn to_rgb16(&self) -> Pixel16 {
        let mut out = Pixel16 { r: 0, g: 0, b: 0 };
        let h_decimal = (self.h * 6.0) - (((self.h * 6.0) as u8) as f32);
        match (self.h * 6.0 % 6.0) as u8 {
            0 => {
                out.r = (self.v * 65535.0) as u16;
                out.g = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 65535.0) as u16;
                out.b = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
            }
            1 => {
                out.r = ((self.v * (1.0 - self.s * h_decimal)) * 65535.0) as u16;
                out.g = (self.v * 65535.0) as u16;
                out.b = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
            }
            2 => {
                out.r = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
                out.g = (self.v * 65535.0) as u16;
                out.b = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 65535.0) as u16;
            }
            3 => {
                out.r = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
                out.g = ((self.v * (1.0 - self.s * h_decimal)) * 65535.0) as u16;
                out.b = (self.v * 65535.0) as u16;
            }
            4 => {
                out.r = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 65535.0) as u16;
                out.g = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
                out.b = (self.v * 65535.0) as u16;
            }
            5 => {
                out.r = (self.v * 65535.0) as u16;
                out.g = ((self.v * (1.0 - self.s)) * 65535.0) as u16;
                out.b = ((self.v * (1.0 - self.s * h_decimal)) * 65535.0) as u16;
            }
            _ => (),
        }
        out
    }
}

pub fn hsv_interp(col1: &PixelHsv, col2: &PixelHsv, pos: f32) -> PixelHsv {
    let h_range = col1.h - col2.h;
    let s_range = col1.s - col2.s;
    let v_range = col1.v - col2.v;
    
    let h_out = col1.h - (h_range * pos.min(1.0));
    let s_out = col1.s - (s_range * pos.min(1.0));
    let v_out = col1.v - (v_range * pos.min(1.0));
    
    PixelHsv { h: h_out, s: s_out, v: v_out }
}
