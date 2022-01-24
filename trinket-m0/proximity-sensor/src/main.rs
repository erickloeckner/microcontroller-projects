#![no_std]
#![no_main]

use panic_halt as _;
use trinket_m0 as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use atsamd_hal::gpio::Pin;
use atsamd_hal::gpio::v2::pin::{Input, Output, PinId, PullDown, PushPull};

struct RingBuffer8 {
    buffer: [u16; 8],
    pos: usize,
}

impl RingBuffer8 {
    fn new() -> RingBuffer8 {
        RingBuffer8 {
            buffer: [0; 8],
            pos: 0,
        }
    }
    
    fn push(&mut self, value: u16) {
        self.buffer[self.pos] = value;
        self.pos = (self.pos + 1) % self.buffer.len();
    }
    
    fn mean(&mut self) -> u16 {
        let sum: u32 = self.buffer
            .iter()
            .fold(0, |s, &i| s + (i as u32));
        (sum / (self.buffer.len() as u32)) as u16
        
    }
}

struct DistanceSensor<T: PinId, E: PinId> {
    trig: Pin<T, Output<PushPull>>,
    echo: Pin<E, Input<PullDown>>,
    wait_us: u16,
    pulse_us: u16,
    timeout: u16,
}

impl<T: PinId, E: PinId> DistanceSensor<T, E> {
    fn get_value(&mut self, delay: &mut Delay) -> u16 {
        self.wait_us = 0;
        self.pulse_us = 0;
        
        self.trig.set_high().unwrap();
        delay.delay_us(100u8);
        self.trig.set_low().unwrap();
        
        while self.echo.is_low().unwrap() {
            delay.delay_us(1u8);
            self.wait_us = self.wait_us.saturating_add(1);
            
            if self.wait_us >= self.timeout {
                break;
            }
        }
        
        while self.echo.is_high().unwrap() {
            delay.delay_us(1u8);
            self.pulse_us = self.pulse_us.saturating_add(1);
            
            if self.pulse_us >= self.timeout {
                break;
            }
        }
        self.pulse_us
    }
}

enum MainState {
    Off,
    On,
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);
    let mut red_led = pins.d13.into_open_drain_output(&mut pins.port);
    
    let mut relay_pin = pins.d2.into_open_drain_output(&mut pins.port);
    relay_pin.set_high().unwrap();
    
    let mut delay = Delay::new(core.SYST, &mut clocks);
    
    let mut sensor1 = DistanceSensor {
        trig: pins.d0.into_open_drain_output(&mut pins.port),
        echo: pins.d1.into_pull_down_input(&mut pins.port),
        wait_us: 0u16,
        pulse_us: 0u16,
        timeout: 65535u16,
    };
    
    let mut buf = RingBuffer8::new();
    
    let mut state = MainState::Off;
    let hold_time: u16 = 60000;
    let mut hold_current: u16 = 0;
    let threshold = 520;
    
    loop {
        match state {
            MainState::Off => {
                buf.push(sensor1.get_value(&mut delay));
                
                if buf.mean() < threshold {
                    red_led.set_high().unwrap();
                    relay_pin.set_low().unwrap();
                    state = MainState::On;
                }
                else {
                    delay.delay_ms(100u16);
                }
            }
            
            MainState::On => {
                buf.push(sensor1.get_value(&mut delay));
                
                if buf.mean() < threshold {
                    hold_current = 0;
                }
                else {
                    hold_current += 100;
                }
                delay.delay_ms(100u16);
                
                if hold_current >= hold_time {
                    hold_current = 0;
                    red_led.set_low().unwrap();
                    relay_pin.set_high().unwrap();
                    state = MainState::Off;
                }
            }
        }
    }
}
