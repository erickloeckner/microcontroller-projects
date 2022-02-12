#![no_std]
#![no_main]

mod buffer;
use crate::buffer::RingBuffer;
mod sensor;
use crate::sensor::DistanceSensor;

use core::cell::Cell;

use bsp::hal;
use panic_halt as _;
use trinket_m0 as bsp;

use bsp::entry;
use cortex_m::interrupt as cortex_interrupt;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals, TC3};
use hal::prelude::*;
use hal::timer::TimerCounter;

const LOOP_TIME: u32 = 100;    // number of milliseconds between sensor updates
const THRESHOLD: u16 = 520;    // minimum value of sensor average reading before switch is activated
const HOLD_TIME: u32 = 300000; // number of milliseconds to keep switch active after sensor is above threshold
const BUFFER_SIZE: usize = 8;  // ring buffer size for storing sensor readings

enum MainState {
    Off,
    On,
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
    
    let mut pins = bsp::Pins::new(peripherals.PORT);
    let mut red_led = pins.d13.into_push_pull_output(&mut pins.port);
    
    let mut relay_pin = pins.d2.into_push_pull_output(&mut pins.port);
    relay_pin.set_high().unwrap();
    
    let mut delay = Delay::new(core.SYST, &mut clocks);
    
    let mut sensor1 = DistanceSensor::new(
        pins.d0.into_push_pull_output(&mut pins.port).into(),
        pins.d1.into_pull_down_input(&mut pins.port).into(),
        65535u16,
    );
    
    let mut buf = RingBuffer::<u16, BUFFER_SIZE>::new();
    
    let mut state = MainState::Off;
    let mut last_update: u32 = 0;
    let mut last_on: u32 = 0;
    
    loop {
        cortex_interrupt::free(|cs| {
            millis = MILLIS.borrow(cs).get();
        });
        
        if millis.wrapping_sub(last_update) >= LOOP_TIME {
            last_update = millis;
            buf.push(sensor1.get_value(&mut delay));
            
            match state {
                MainState::Off => {
                    if buf.mean() < THRESHOLD {
                        red_led.set_high().unwrap();
                        relay_pin.set_low().unwrap();
                        last_on = millis;
                        state = MainState::On;
                    }
                }
                MainState::On => {
                    if buf.mean() < THRESHOLD {
                        last_on = millis;
                    }
                    
                    if millis.wrapping_sub(last_on) >= HOLD_TIME {
                        red_led.set_low().unwrap();
                        relay_pin.set_high().unwrap();
                        state = MainState::Off;
                    }
                }
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
