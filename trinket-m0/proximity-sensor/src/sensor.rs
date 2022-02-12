use crate::hal::delay::Delay;
use crate::hal::gpio::v2::{Input, Output, Pin, PinId, PullDown, PushPull};
use crate::hal::prelude::*;

pub struct DistanceSensor<T: PinId, E: PinId> {
    trig: Pin<T, Output<PushPull>>,
    echo: Pin<E, Input<PullDown>>,
    timeout: u16,
}

impl<T: PinId, E: PinId> DistanceSensor<T, E> {
    pub fn new(trig: Pin<T, Output<PushPull>>, echo: Pin<E, Input<PullDown>>, timeout: u16) -> Self {
        DistanceSensor { trig: trig, echo: echo, timeout: timeout }
    }
    
    pub fn get_value(&mut self, delay: &mut Delay) -> u16 {
        let mut wait_us: u16 = 0;
        let mut pulse_us: u16 = 0;
        
        self.trig.set_high().unwrap();
        delay.delay_us(100u8);
        self.trig.set_low().unwrap();
        
        while self.echo.is_low().unwrap() {
            delay.delay_us(1u8);
            wait_us = wait_us.saturating_add(1);
            if wait_us >= self.timeout {
                break;
            }
        }
        
        while self.echo.is_high().unwrap() {
            delay.delay_us(1u8);
            pulse_us = pulse_us.saturating_add(1);
            if pulse_us >= self.timeout {
                break;
            }
        }
        pulse_us
    }
}
