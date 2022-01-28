#![no_std]
#![no_main]

mod buffer;
mod prng;
mod utilities;

use core::cell::Cell;

use crate::buffer::RingBuffer8;
use crate::prng::Prng;
use crate::utilities::*;

use bsp::hal;
use panic_halt as _;
use trinket_m0 as bsp;

use atsamd_hal::gpio::v2::AlternateB;
use atsamd_hal::thumbv6m::adc::SampleRate;
use bsp::entry;
use hal::adc::Adc;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals, TC3};
use hal::prelude::*;
use hal::timer::TimerCounter;
use trinket_m0::spi_master;
use cortex_m::interrupt as cortex_interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;

const NUM_LEDS: usize = 62;
const BRIGHTNESS: u8 = 63;
const BUF_SIZE: usize = 4 + (NUM_LEDS * 4) + ((NUM_LEDS + 1) / 2);
const RAND_SCALE: u8 = 8;
const SINE_SPEED: u8 = 100;
const ADC_MIN: u16 = 500;
const ADC_MAX: u16 = 4095;
const ADC_SCALE_NUMER: u16 = 5;
const ADC_SCALE_DENOM: u16 = 4;
const MS_PER_FRAME: u32 = 10;

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
        if index < self.len {
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
    }
    
    fn fill_gradient(&mut self, start: PixelHsv, end: PixelHsv) {
        let len = self.len;
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            if i < len {
                let rgb = hsv_2_rgb(hsv_interp(start, end, map_range(i as u8, (len - 1) as u8, 255)));
                
                v[0] = 255;
                v[1] = rgb.b;
                v[2] = rgb.g;
                v[3] = rgb.r;
            }
        });
    }
    
    fn fill_gradient_mod(&mut self, start: PixelHsv, end: PixelHsv, rand: &[RandomSprite; NUM_LEDS]) {
        let len = self.len;
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            if i < len {
                let mut hsv = hsv_interp(start, end, map_range(i as u8, (len - 1) as u8, 255));
                hsv.h = hsv.h.wrapping_add(rand[i].value);
                //~ let rgb = hsv_2_rgb(hsv_interp(start, end, map_range(i as u8, (len - 1) as u8, 255)));
                let rgb = hsv_2_rgb(hsv);
                
                v[0] = 255;
                v[1] = rgb.b;
                v[2] = rgb.g;
                v[3] = rgb.r;
            }
        });
    }
    
    fn fill_sine(&mut self, start: PixelHsv, end: PixelHsv, sin: &[u8; 256], offset: u8) {
        let len = self.len;
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            if i < len {
                let table_index = map_range(i as u8, (NUM_LEDS - 1) as u8, 255).wrapping_add(offset);
                let hsv = hsv_interp(start, end, sin[table_index as usize]);

                let rgb = hsv_2_rgb(hsv);
                
                v[0] = 255;
                v[1] = rgb.b;
                v[2] = rgb.g;
                v[3] = rgb.r;
            }
        });
    }
    
    fn fill_sine_mod(&mut self, start: PixelHsv, end: PixelHsv, sin: &[u8; 256], offset: u8, rand: &[RandomSprite; NUM_LEDS]) {
        let len = self.len;
        self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
            if i < len {
                let table_index = map_range(i as u8, (NUM_LEDS - 1) as u8, 255).wrapping_add(offset);
                let rand_bi = (rand[i].value as i16) - ((rand[i].scale as i16) / 2);
                
                let mut hsv = hsv_interp(start, end, sin[table_index as usize]);
                match rand_bi {
                    -32768..=-1 => { hsv.h = hsv.h.wrapping_sub((rand_bi * -1) as u8); }
                    0 => (),
                    1..=32767 => { hsv.h = hsv.h.wrapping_add(rand_bi as u8); }
                }
                let rgb = hsv_2_rgb(hsv);
                
                v[0] = 255;
                v[1] = rgb.b;
                v[2] = rgb.g;
                v[3] = rgb.r;
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
        h_out = col1.h - map_range(value, 255, h_delta);
    } else {
        let h_delta = col2.h - col1.h;
        h_out = col1.h + map_range(value, 255, h_delta);
    }
    
    if col1.s == col2.s {
        s_out = col1.s;
    } else if col1.s > col2.s {
        s_out = col1.s - map_range(value, 255, col1.s - col2.s);
    } else {
        s_out = col1.s + map_range(value, 255, col2.s - col1.s);
    }
    
    if col1.v == col2.v {
        v_out = col1.v;
    } else if col1.v > col2.v {
        v_out = col1.v - map_range(value, 255, col1.v - col2.v);
    } else {
        v_out = col1.v + map_range(value, 255, col2.v - col1.v);
    }
    
    PixelHsv { h: h_out, s: s_out, v: v_out }
}

#[derive(Debug, Copy, Clone)]
struct RandomSprite {
    value: u8,
    target: u8,
    scale: u8,
    step: u8,
    timer: u8,
}

impl RandomSprite {
    fn new(scale: u8, rng: &mut Prng) -> Self {
        let value = u8_scale(rng.rand() as u8, scale);
        let target = u8_scale(rng.rand() as u8, scale);
        let step = u8_scale(rng.rand() as u8, 50) + 150;
        
        RandomSprite {
            value: value,
            target: target,
            scale: scale,
            step: step,
            timer: 0,
        }
    }
    
    fn run(&mut self, rng: &mut Prng) {
        self.timer += 1;
        if self.timer >= self.step {
            if self.value > self.target {
                self.value -= 1;
                self.timer = 0;
            } else if self.value < self.target {
                self.value += 1;
                self.timer = 0;
            } else {
                self.target = u8_scale(rng.rand() as u8, self.scale);
                self.step = u8_scale(rng.rand() as u8, 50) + 150;
                self.timer = 0;
            }
        }
    }
}

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = bsp::Pins::new(peripherals.PORT);
    
    //~ let mut delay = Delay::new(core.SYST, &mut clocks);
    let gclk0 = clocks.gclk0();
    let tc23 = &clocks.tcc2_tc3(&gclk0).unwrap();
    
    unsafe {
        core.NVIC.set_priority(interrupt::TC3, 1);
        NVIC::unmask(interrupt::TC3);
    }
    
    let mut timer = TimerCounter::tc3_(tc23, peripherals.TC3, &mut peripherals.PM);
    timer.start(1.ms());
    timer.enable_interrupt();
    let mut millis: u32 = 0;
    
    let mut spi = spi_master(
        &mut clocks,
        10.mhz(),
        peripherals.SERCOM0,
        &mut peripherals.PM,
        pins.d3,
        pins.d4,
        pins.d2,
        &mut pins.port,
    );
    
    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    adc.samples(SampleRate::_32);
    let mut a0 = pins.d0.into_mode::<AlternateB>();
    let mut a1 = pins.d1.into_mode::<AlternateB>();
    let mut pot1_buffer = RingBuffer8::new();
    let mut pot2_buffer = RingBuffer8::new();
    
    let mut rng = Prng::new(123456789);
    let mut random_sprites = [RandomSprite::new(RAND_SCALE, &mut rng); NUM_LEDS];
    
    let sine_table = [127, 130, 133, 137, 140, 143, 146, 149, 152, 155, 158, 161, 164, 167, 170, 173, 176, 179, 182, 185, 187, 190, 193, 196, 198, 201, 203, 206, 208, 211, 213, 215,
        218, 220, 222, 224, 226, 228, 230, 232, 233, 235, 237, 238, 240, 241, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 252, 253, 253, 254, 254, 254, 255, 255,
        255, 255, 255, 254, 254, 254, 253, 253, 252, 251, 251, 250, 249, 248, 247, 246, 245, 243, 242, 241, 239, 238, 236, 234, 233, 231, 229, 227, 225, 223, 221, 219,
        216, 214, 212, 209, 207, 205, 202, 200, 197, 194, 192, 189, 186, 183, 180, 178, 175, 172, 169, 166, 163, 160, 157, 154, 151, 147, 144, 141, 138, 135, 132, 129,
        126, 122, 119, 116, 113, 110, 107, 104, 101, 98, 95, 91, 88, 85, 83, 80, 77, 74, 71, 68, 65, 63, 60, 57, 55, 52, 50, 47, 45, 42, 40, 38,
        36, 33, 31, 29, 27, 25, 24, 22, 20, 18, 17, 15, 14, 12, 11, 10, 9, 7, 6, 5, 4, 4, 3, 2, 2, 1, 1, 1, 1, 0, 0, 0,
        0, 0, 0, 1, 1, 1, 1, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13, 14, 16, 18, 19, 21, 23, 24, 26, 28, 30, 32, 35, 37,
        39, 41, 44, 46, 48, 51, 53, 56, 59, 61, 64, 67, 70, 72, 75, 78, 81, 84, 87, 90, 93, 96, 99, 102, 105, 108, 111, 115, 118, 121, 124, 127];

    let mut sine_offset: u8 = 0;
    let mut sine_offset_timer: u8 = 0;
    let mut last_frame = 0;
    let mut dt: u32;
    let pattern: u8 = 0;
    let mut pot1: u16;
    let mut pot1_adj: u16;
    let mut pot2: u16;
    let mut pot2_adj: u16;
    let mut color1: u8;
    let mut color2: u8;
    
    let mut leds = Leds::new();
    leds.fill_buffer();
    let _ = spi.write(&leds.buffer[..]);
    
    loop {        
        cortex_interrupt::free(|cs| {
            millis = MILLIS.borrow(cs).get();
        });
        dt = ((millis as i64) - (last_frame as i64)).abs() as u32;
        
        if dt >= MS_PER_FRAME {
            last_frame = millis;
            
            for i in random_sprites.iter_mut() {
                i.run(&mut rng);
            }
            
            pot1 = adc.read(&mut a0).unwrap();
            pot2 = adc.read(&mut a1).unwrap();
            
            pot1_adj = ((pot1.saturating_sub(ADC_MIN)) * ADC_SCALE_NUMER / ADC_SCALE_DENOM).min(ADC_MAX);
            pot2_adj = ((pot2.saturating_sub(ADC_MIN)) * ADC_SCALE_NUMER / ADC_SCALE_DENOM).min(ADC_MAX);
            
            pot1_buffer.push((pot1_adj >> 4) as u8);
            pot2_buffer.push((pot2_adj >> 4) as u8);
            
            color1 = pot1_buffer.mean();
            color2 = pot2_buffer.mean();
            
            match pattern {
                0 => { leds.fill_gradient(PixelHsv { h: color1, s: 255, v: BRIGHTNESS }, PixelHsv { h: color2, s: 255, v: BRIGHTNESS }); }
                1 => { leds.fill_gradient_mod(PixelHsv { h: color1, s: 255, v: BRIGHTNESS }, PixelHsv { h: color2, s: 255, v: BRIGHTNESS }, &random_sprites); }
                2 => { leds.fill_sine(PixelHsv { h: color1, s: 255, v: BRIGHTNESS }, PixelHsv { h: color2, s: 255, v: BRIGHTNESS }, &sine_table, sine_offset); }
                3 => { leds.fill_sine_mod(PixelHsv { h: color1, s: 255, v: BRIGHTNESS }, PixelHsv { h: color2, s: 255, v: BRIGHTNESS }, &sine_table, sine_offset, &random_sprites); }
                _ => (),
            }
            let _ = spi.write(&leds.buffer[..]);
            
            sine_offset_timer = sine_offset_timer.wrapping_add(1);
            if sine_offset_timer >= SINE_SPEED {
                sine_offset_timer = 0;
                sine_offset = sine_offset.wrapping_add(1);
            }
        }
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
