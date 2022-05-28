#![no_std]
#![no_main]

mod prng;
use crate::prng::Prng;

use core::cell::Cell;
use core::slice::Iter;

use feather_m4 as bsp;
#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::entry;
use bsp::hal;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, interrupt, Peripherals, TC3};
use hal::prelude::*;
use hal::timer::TimerCounter;
use cortex_m::interrupt as cortex_interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;

const NUM_LEDS: usize = 10;
const BUF_SIZE: usize = 4 + (NUM_LEDS * 4) + ((NUM_LEDS + 1) / 2);
const NUM_SPRITES: usize = 2;

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[derive(Debug, Copy, Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Copy, Clone)]
struct PixelHsv {
    h: u8,
    s: u8,
    v: u8,
}

struct Leds {
    len: usize,
    buffer: [u8; BUF_SIZE],
}

impl Leds {
    fn new() -> Self {
        Leds {
            len: NUM_LEDS,
            buffer: [0; BUF_SIZE],
        }
    }
    
    fn fill_buffer(&mut self) {
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
    
    fn set_led(&mut self, pixel: Pixel, index: usize) {
        match self.buffer.chunks_mut(4).skip(1).nth(index) {
            Some(v) => {
                v[0] = 255;
                v[1] = pixel.b;
                v[2] = pixel.g;
                v[3] = pixel.r;
            },
            None => {},
        }
    }
    
    fn fill_gradient(&mut self, start: PixelHsv, end: PixelHsv) {
        let len = self.len;
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            if i < len {
                let rgb = hsv_2_rgb(hsv_interp(start, end, u8_map_range(i as u8, (len - 1) as u8, 255)));
                
                v[0] = 255;
                v[1] = rgb.b;
                v[2] = rgb.g;
                v[3] = rgb.r;
            }
        });
    }
    
    //~ fn fill_sprite(&mut self, sprite: &Sprite, fg_color: &PixelHsv, bg_color: &PixelHsv) {
        //~ let len = self.len;
        //~ self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            //~ if i < len {
                //~ let rgb = hsv_2_rgb(hsv_interp(start, end, u8_map_range(i as u8, (len - 1) as u8, 255)));
                //~ let pos = u16_map_range(i as u16, (len - 1) as u16, u16::MAX);
                //~ let delta = ((pos as i32) - (sprite.pos as i32)).abs() as u16;
                //~ let mix = ((u16::MAX as u32) - ((delta as u32) * (sprite.falloff as u32)).min(u16::MAX as u32)) as u16;
                //~ let mix_8 = (mix >> 8) as u8;
                //~ let rgb = hsv_2_rgb(hsv_interp(*bg_color, *fg_color, mix_8));
                
                //~ v[0] = 255;
                //~ v[1] = rgb.b;
                //~ v[2] = rgb.g;
                //~ v[3] = rgb.r;
            //~ }
        //~ });
    //~ }
    
    fn fill_sprites(&mut self, sprites: Iter<Sprite>, fg_color: &PixelHsv, bg_color: &PixelHsv) {
        let len = self.len;
        let mut gradient = [(0u8, 0u16); NUM_LEDS];
        gradient.iter_mut().enumerate().for_each(|(index, value)| {
            value.1 = u16_map_range(index as u16, (NUM_LEDS - 1) as u16, u16::MAX);
        });
        
        for sprite in sprites {
            gradient.iter_mut().for_each(|i| {
                //~ let pos = u16_map_range(i as u16, (NUM_LEDS - 1) as u16, u16::MAX);
                let delta = ((i.1 as i32) - (sprite.pos as i32)).abs() as u16;
                let mix = ((u16::MAX as u32) - ((delta as u32) * (sprite.falloff as u32)).min(u16::MAX as u32)) as u16;
                let mix_8 = (mix >> 8) as u8;
                i.0 = i.0.saturating_add(mix_8);
            });
        }
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(index, value)| {
            if index < len {
                let rgb = hsv_2_rgb(hsv_interp(*bg_color, *fg_color, gradient[index].0));
                
                value[0] = 255;
                value[1] = rgb.b;
                value[2] = rgb.g;
                value[3] = rgb.r;
            }
        });
    }
}

#[derive(Copy, Clone)]
struct Sprite {
    pos: u16,
    falloff: u16,
    speed: i8,
}

struct SpriteGroup<const N: usize> {
    sprites: [Sprite; N],
    speed_min: u8,
    speed_range: u8,
}

impl <const N: usize> SpriteGroup<N> {
    fn new(falloff: u16, speed_min: u8, speed_range: u8, rng: &mut Prng) -> Self {
        let speed_min_clip = speed_min.max(1);
        let speed_range_clip = speed_range.max(1);
        
        let mut sprites = [Sprite { pos: 0, falloff: falloff, speed: 1 }; N];
        for i in sprites.iter_mut() {
            i.pos = rng.rand() as u16;
            let speed_rand = u8_scale(rng.rand() as u8, speed_range_clip);
            i.speed = (speed_min_clip + speed_rand).min(i8::MAX as u8) as i8;
            //~ if i.speed == 0 { i.speed = 1 }
        }
        Self { sprites: sprites, speed_min: speed_min_clip, speed_range: speed_range_clip }
    }
    
    fn sprites_iter(&self) -> Iter<Sprite> {
        self.sprites.iter()
    }
    
    fn run(&mut self, dt: u32, rng: &mut Prng) {
        self.sprites.iter_mut().for_each(|i| {
            let speed = (dt as i8) * i.speed;
            if speed >= 1 {
                i.pos = i.pos.saturating_add(speed as u16);
                if i.pos == u16::MAX { 
                    //~ i.speed = i.speed * -1;
                    let speed_rand = u8_scale(rng.rand() as u8, self.speed_range);
                    i.speed = ((self.speed_min + speed_rand).min(i8::MAX as u8) as i8) * -1;
                }
            }
            if speed <= -1 {
                i.pos = i.pos.saturating_sub((speed * -1) as u16);
                if i.pos == u16::MIN { 
                    //~ i.speed = i.speed * -1;
                    let speed_rand = u8_scale(rng.rand() as u8, self.speed_range);
                    i.speed = (self.speed_min + speed_rand).min(i8::MAX as u8) as i8;
                }
            }
        });
    }
}

fn hsv_2_rgb(pixel: PixelHsv) -> Pixel {
    if pixel.s == 0 {
        Pixel {r: pixel.v, g: pixel.v, b: pixel.v}
    } else {
        let i = pixel.h / 43;
        let f = pixel.h % 43 * 6;
        let p = u8_scale(pixel.v, 255 - pixel.s);
        let q = u8_scale(pixel.v, 255 - u8_scale(pixel.s, f));
        let t = u8_scale(pixel.v, 255 - u8_scale(pixel.s, 255 - f));

        match i {
            0 => {
                Pixel {r: pixel.v, g: t, b: p}
            }
            1 => {
                Pixel {r: q, g: pixel.v, b: p}
            }
            2 => {
                Pixel {r: p, g: pixel.v, b: t}
            }
            3 => {
                Pixel {r: p, g: q, b: pixel.v}
            }
            4 => {
                Pixel {r: t, g: p, b: pixel.v}
            }
            5 => {
                Pixel {r: pixel.v, g: p, b: q}
            }
            _ => {
                Pixel {r: 0, g: 0, b: 0}
            }
        }
    }
}

fn hsv_interp(col1: PixelHsv, col2: PixelHsv, value: u8) -> PixelHsv {
    let h_out: u8;
    let s_out: u8;
    let v_out: u8;
    
    if col1.h == col2.h {
        h_out = col1.h;
    } else if col1.h > col2.h {
        let h_delta = col1.h - col2.h;
        h_out = col1.h - u8_map_range(value, 255, h_delta);
    } else {
        let h_delta = col2.h - col1.h;
        h_out = col1.h + u8_map_range(value, 255, h_delta);
    }
    
    if col1.s == col2.s {
        s_out = col1.s;
    } else if col1.s > col2.s {
        s_out = col1.s - u8_map_range(value, 255, col1.s - col2.s);
    } else {
        s_out = col1.s + u8_map_range(value, 255, col2.s - col1.s);
    }
    
    if col1.v == col2.v {
        v_out = col1.v;
    } else if col1.v > col2.v {
        v_out = col1.v - u8_map_range(value, 255, col1.v - col2.v);
    } else {
        v_out = col1.v + u8_map_range(value, 255, col2.v - col1.v);
    }
    
    PixelHsv { h: h_out, s: s_out, v: v_out }
}

fn u8_scale(val: u8, scale: u8) -> u8 {
    ((val as u16 * (scale as u16 + 1)) >> 8) as u8
}

fn u16_scale(val: u16, scale: u16) -> u16 {
    ((val as u32 * (scale as u32 + 1)) >> 16) as u16
}

fn u32_scale(val: u32, scale: u32) -> u32 {
    ((val as u64 * (scale as u64 + 1)) >> 32) as u32
}

fn u8_map_range(input: u8, input_max: u8, output_max: u8) -> u8 {
    let percent_in = ((input as u16 * 255) / (input_max as u16)) as u8;
    u8_scale(output_max, percent_in)
}

fn u16_map_range(input: u16, input_max: u16, output_max: u16) -> u16 {
    let percent_in = ((input as u32 * 65535) / (input_max as u32)) as u16;
    u16_scale(output_max, percent_in)
}

//~ fn map_range(input: u8, input_max: u8, output_max: u8) -> u8 {
    //~ let percent_in = ((input as u16 * 255) / (input_max as u16)) as u8;
    //~ u8_scale(output_max, percent_in)
//~ }

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    
    // begin interrupt setup
    let gclk0 = clocks.gclk0();
    let tc23 = &clocks.tc2_tc3(&gclk0).unwrap();
    unsafe {
        core.NVIC.set_priority(interrupt::TC3, 1);
        NVIC::unmask(interrupt::TC3);
    }
    let mut timer = TimerCounter::tc3_(tc23, peripherals.TC3, &mut peripherals.MCLK);
    timer.start(1.ms());
    timer.enable_interrupt();
    let mut millis: u32 = 0;
    // end interrupt setup
    
    let mut delay = Delay::new(core.SYST, &mut clocks);
    
    let mut red_led = pins.d13.into_push_pull_output();
    
    let mut spi = feather_m4::spi_master(
        &mut clocks,
        10.mhz(),
        peripherals.SERCOM1,
        &mut peripherals.MCLK,
        pins.sck,
        pins.mosi,
        pins.miso,
    );
    
    let mut rng = Prng::new(123456789);
    //~ let mut sprite = Sprite { pos: 0, falloff: 4, speed: 10 };
    let mut sprites = SpriteGroup::<NUM_SPRITES>::new(3, 1, 8, &mut rng);
    //~ sprites.
    
    let mut leds = Leds::new();
    leds.fill_buffer();
    
    //~ leds.fill_gradient(PixelHsv { h: 120, s: 255, v: 63 }, PixelHsv { h: 200, s: 255, v: 63 });
    //~ leds.fill_sprite(&sprite, &PixelHsv { h: 0, s: 255, v: 31 }, &PixelHsv { h: 63, s: 255, v: 31 });
    //~ leds.fill_sprites(sprites.sprites_iter(), &PixelHsv { h: 0, s: 255, v: 31 }, &PixelHsv { h: 63, s: 255, v: 31 });
    //~ let _ = spi.write(&leds.buffer[..]);
    
    loop {
        cortex_interrupt::free(|cs| {
            millis = MILLIS.borrow(cs).get();
        });
        
        //~ sprite.pos = sprite.pos.wrapping_add(10);
        sprites.run(1, &mut rng);
        //~ leds.fill_sprite(&sprite, &PixelHsv { h: 0, s: 255, v: 31 }, &PixelHsv { h: 63, s: 255, v: 31 });
        leds.fill_sprites(sprites.sprites_iter(), &PixelHsv { h: 170, s: 255, v: 31 }, &PixelHsv { h: 127, s: 255, v: 31 });
        let _ = spi.write(&leds.buffer[..]);
        delay.delay_ms(10u8);
    }
}

#[interrupt]
fn TC3() {
    cortex_interrupt::free(|cs| MILLIS.borrow(cs).set(MILLIS.borrow(cs).get().wrapping_add(1)));
    
    unsafe {
        TC3::ptr()
            .as_ref()
            .unwrap()
            .count16()
            .intflag
            .modify(|_, w| w.ovf().set_bit());
    }
}
