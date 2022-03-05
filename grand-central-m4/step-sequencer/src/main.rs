#![no_std]
#![no_main]

mod button;
use crate::button::Button;
mod buttonset;
use crate::buttonset::{ChannelButtonSet,StepButtonSet};
mod midi;

use core::cell::Cell;

use bsp::hal;
use grand_central_m4 as bsp;

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::entry;
use cortex_m::interrupt as cortex_interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, interrupt, Peripherals, TC3};
use hal::prelude::*;
use hal::sercom::v2::spi::MODE_0;
use hal::sercom::v2::{IoSet2, Sercom0, uart::{self, EightBit, Parity, StopBits}};
use hal::sercom::{PadPin, SPIMaster7};
//~ use hal::sercom::{PadPin, SPIMaster7};
use hal::timer::TimerCounter;

const NUM_LEDS: usize = 16;
const BUF_SIZE: usize = 4 + (NUM_LEDS * 4) + ((NUM_LEDS + 1) / 2);
const NUM_STEPS: usize = 32;   // max number of steps per pattern
const NUM_CHANNELS: usize = 8; // number of pattern channels
const US_PER_UPDATE_LEDS: u32 = 10000;  // microseconds between LED updates
const CHANNEL_NOTES: [u8; NUM_CHANNELS] = [36, 37, 38, 39, 40, 41, 42, 43];
const VELOCITY_LEVELS: [u8; 4] = [31, 63, 95, 127];

static MICROS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

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
}

#[derive(Debug, Copy, Clone)]
pub struct Step {
    gate: bool,
    vel: u8,
    start: f32,
    duration: f32,
    note_on_sent: bool,
    note_off_sent: bool,
}

pub struct StepTimer {
    num_steps: usize,
    step: usize,
    step_progress: f32,
    time_per_step: u32,
    last_step_time: u32,
    run_state: bool,
}

impl StepTimer {
    fn new(num_steps: usize, time_per_step: u32) -> Self {
        StepTimer {
            num_steps: num_steps,
            step: 0,
            step_progress: 0.0,
            time_per_step: time_per_step,
            last_step_time: 0,
            run_state: false,
        }
    }
    
    fn start(&mut self, time: u32) {
        self.run_state = true;
        self.last_step_time = time;
    }
    
    fn stop(&mut self) {
        self.step = 0;
        self.run_state = false;
    }
    
    fn get_step(&self) -> usize {
        self.step
    }
    
    fn get_step_progress(&self) -> f32 {
        self.step_progress
    }
    
    fn get_run_state(&self) -> bool {
        self.run_state
    }
    
    fn poll(&mut self, time: u32, steps: &mut [[Step; NUM_STEPS]; NUM_CHANNELS]) {
        if self.run_state == true {
            self.step_progress = (time.wrapping_sub(self.last_step_time) as f32) / (self.time_per_step as f32);
            
            if time.wrapping_sub(self.last_step_time) >= self.time_per_step {
                for channel in steps.iter_mut() {
                    channel[self.step].note_on_sent = false;
                    channel[self.step].note_off_sent = false;
                }
                
                self.step = (self.step + 1) % self.num_steps;
                self.step_progress = 0.0;
                //~ self.last_step_time = self.last_step_time.wrapping_add(self.time_per_step);
                self.last_step_time = time;
            }
        }
    }
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let gclk0 = clocks.gclk0();
    let tc23 = &clocks.tc2_tc3(&gclk0).unwrap();
    unsafe {
        core.NVIC.set_priority(interrupt::TC3, 1);
        NVIC::unmask(interrupt::TC3);
    }
    let mut timer = TimerCounter::tc3_(tc23, peripherals.TC3, &mut peripherals.MCLK);
    timer.start(1.us());
    timer.enable_interrupt();
    let mut micros: u32 = 0;
    
    let mut delay = Delay::new(core.SYST, &mut clocks);
    //~ delay.delay_ms(400u16);

    let mut pins = bsp::Pins::new(peripherals.PORT);
    let mut red_led = pins.red_led.into_open_drain_output(&mut pins.port);
    
    let mut spi = SPIMaster7::new(
        &mut clocks.sercom7_core(&gclk0).unwrap(),
        10.mhz(),
        MODE_0,
        peripherals.SERCOM7,
        &mut peripherals.MCLK,
        (pins.miso.into_pad(&mut pins.port), pins.mosi.into_pad(&mut pins.port), pins.sck.into_pad(&mut pins.port)),
    );
    
    let uart_clock = &clocks.sercom0_core(&gclk0).unwrap();
    let uart_pads = uart::Pads::<Sercom0, IoSet2>::default()
        .rx(pins.uart0_rx)
        .tx(pins.uart0_tx);
    let mut uart = uart::Config::new(&mut peripherals.MCLK, peripherals.SERCOM0, uart_pads, uart_clock.freq())
        .baud(31251.hz(), uart::BaudMode::Fractional(uart::Oversampling::Bits16))
        //~ .char_size::<EightBit>()
        //~ .parity(Parity::None)
        //~ .stop_bits(StopBits::OneBit)
        .enable();

    let mut leds = Leds::new();
    leds.fill_buffer();
    let _ = spi.write(&leds.buffer[..]);
    
    let mut last_update_leds: u32 = 0;

    let mut step_buttons = StepButtonSet { 
        b00: Button::new(pins.d32.into_pull_up_input(&mut pins.port).into(), 1000),
        b01: Button::new(pins.d33.into_pull_up_input(&mut pins.port).into(), 1000),
        b02: Button::new(pins.d34.into_pull_up_input(&mut pins.port).into(), 1000),
        b03: Button::new(pins.d35.into_pull_up_input(&mut pins.port).into(), 1000),
        b04: Button::new(pins.d36.into_pull_up_input(&mut pins.port).into(), 1000),
        b05: Button::new(pins.d37.into_pull_up_input(&mut pins.port).into(), 1000),
        b06: Button::new(pins.d38.into_pull_up_input(&mut pins.port).into(), 1000),
        b07: Button::new(pins.d39.into_pull_up_input(&mut pins.port).into(), 1000),
        b08: Button::new(pins.d40.into_pull_up_input(&mut pins.port).into(), 1000),
        b09: Button::new(pins.d41.into_pull_up_input(&mut pins.port).into(), 1000),
        b10: Button::new(pins.d42.into_pull_up_input(&mut pins.port).into(), 1000),
        b11: Button::new(pins.d43.into_pull_up_input(&mut pins.port).into(), 1000),
        b12: Button::new(pins.d44.into_pull_up_input(&mut pins.port).into(), 1000),
        b13: Button::new(pins.d45.into_pull_up_input(&mut pins.port).into(), 1000),
        b14: Button::new(pins.d46.into_pull_up_input(&mut pins.port).into(), 1000),
        b15: Button::new(pins.d47.into_pull_up_input(&mut pins.port).into(), 1000),
        
        v00: Button::new(pins.d4.into_pull_up_input(&mut pins.port).into(), 1000),
        v01: Button::new(pins.d5.into_pull_up_input(&mut pins.port).into(), 1000),
        v02: Button::new(pins.d6.into_pull_up_input(&mut pins.port).into(), 1000),
        v03: Button::new(pins.d7.into_pull_up_input(&mut pins.port).into(), 1000),
    };
    
    let mut channel_buttons = ChannelButtonSet { 
        p00: Button::new(pins.uart3_tx.into_pull_up_input(&mut pins.port).into(), 1000),
        p01: Button::new(pins.uart3_rx.into_pull_up_input(&mut pins.port).into(), 1000),
        p02: Button::new(pins.uart2_tx.into_pull_up_input(&mut pins.port).into(), 1000),
        p03: Button::new(pins.uart2_rx.into_pull_up_input(&mut pins.port).into(), 1000),
        p04: Button::new(pins.uart1_tx.into_pull_up_input(&mut pins.port).into(), 1000),
        p05: Button::new(pins.uart1_rx.into_pull_up_input(&mut pins.port).into(), 1000),
        p06: Button::new(pins.d22.into_pull_up_input(&mut pins.port).into(), 1000),
        p07: Button::new(pins.d23.into_pull_up_input(&mut pins.port).into(), 1000),
    };
    
    let mut start_button = Button::new(pins.d12.into_pull_up_input(&mut pins.port).into(), 1000);
    let mut stop_button = Button::new(pins.d11.into_pull_up_input(&mut pins.port).into(), 1000);
    
    //~ let mut steps = [false; NUM_STEPS];
    let init_step = Step { 
        gate: false,
        vel: 127,
        start: 0.0,
        duration: 0.25,
        note_on_sent: false,
        note_off_sent: false,
    };
    let mut steps = [[init_step; NUM_STEPS]; NUM_CHANNELS];
    
    let mut step_page: usize = 0;    // which set of 16 steps to display on LEDs
    let mut step_channel: usize = 0; // which channel or row is displayed
    
    let mut step_timer = StepTimer::new(NUM_STEPS, 125000);
    //~ step_timer.start(micros);
    
    // 16th/step -- 1.0 / (tempo / 60.0) / 4.0 * 1,000,000
    // 32nd/step -- 1.0 / (tempo / 60.0) / 8.0 * 1,000,000

    loop {
        cortex_interrupt::free(|cs| {
            micros = MICROS.borrow(cs).get();
        });
        
        let step_offset = step_page * NUM_LEDS;
        
        step_buttons.poll(micros);
        step_buttons.update_steps(&mut steps, step_offset, step_channel);
        
        channel_buttons.poll(micros);
        channel_buttons.update_channel(&mut step_channel);
        
        start_button.poll(micros);
        if start_button.rising_edge() == true {
            step_timer.start(micros);
        }
        stop_button.poll(micros);
        if stop_button.rising_edge() == true {
            step_timer.stop();
            
            let current_step = step_timer.get_step();
            // send a note off on all channels' active steps to stop hung notes
            for (index, channel) in steps.iter_mut().enumerate() {
                if channel[current_step].note_on_sent == true && channel[current_step].note_off_sent == false {
                    midi::note_on(&mut uart, 1, CHANNEL_NOTES[index], 0);
                    channel[current_step].note_off_sent = true;
                }
            }    
        }
        
        step_timer.poll(micros, &mut steps);
        
        if step_timer.get_run_state() == true {
            let current_step = step_timer.get_step();
            let current_progress = step_timer.get_step_progress();
            //~ let mut active_notes = false;
            for (index, channel) in steps.iter_mut().enumerate() {
                if channel[current_step].gate == true {
                    if current_progress >= channel[current_step].start && channel[current_step].note_on_sent == false {
                        // send midi note on
                        midi::note_on(&mut uart, 1, CHANNEL_NOTES[index], channel[current_step].vel);
                        channel[current_step].note_on_sent = true;
                    }
                    
                    if current_progress >= channel[current_step].duration && channel[current_step].note_off_sent == false {
                        // send midi note off
                        midi::note_on(&mut uart, 1, CHANNEL_NOTES[index], 0);
                        channel[current_step].note_off_sent = true;
                    }
                }
            }
        }
        
        for (index, step) in steps[step_channel].iter().skip(step_offset).take(NUM_LEDS).enumerate() {
            let actual_step = index + step_offset;
            if step.gate == true {
                // step is enabled and playback cursor is at the step
                if step_timer.get_step() == actual_step && step_timer.get_run_state() == true {
                    leds.set_led(Pixel { r: 95, g: 31, b: 31 }, index);
                // step is enabled
                } else {
                    //~ leds.set_led(Pixel { r: 63, g: 0, b: 0 }, index);
                    
                    if step.vel == VELOCITY_LEVELS[0] { leds.set_led(Pixel { r:  0, g:  0, b: 63 }, index) }
                    if step.vel == VELOCITY_LEVELS[1] { leds.set_led(Pixel { r:  0, g: 63, b:  0 }, index) }
                    if step.vel == VELOCITY_LEVELS[2] { leds.set_led(Pixel { r: 63, g: 63, b:  0 }, index) }
                    if step.vel == VELOCITY_LEVELS[3] { leds.set_led(Pixel { r: 63, g:  0, b:  0 }, index) }
                }
            } else {
                // step is disabled and playback cursor is at the step
                if step_timer.get_step() == actual_step && step_timer.get_run_state() == true {
                    leds.set_led(Pixel { r: 31, g: 31, b: 31 }, index);
                // step is disabled
                } else {
                    leds.set_led(Pixel { r: 0, g: 0, b: 0 }, index);
                }
            }
        }
        
        if micros.wrapping_sub(last_update_leds) >= US_PER_UPDATE_LEDS {
            last_update_leds = micros;
            let _ = spi.write(&leds.buffer[..]);
        }
    }
}

#[interrupt]
fn TC3() {
    cortex_interrupt::free(|cs| MICROS.borrow(cs).set(MICROS.borrow(cs).get().wrapping_add(1)));
    unsafe {
        TC3::ptr()
            .as_ref()
            .unwrap()
            .count16()
            .intflag
            .modify(|_, w| w.ovf().set_bit());
        }
}
