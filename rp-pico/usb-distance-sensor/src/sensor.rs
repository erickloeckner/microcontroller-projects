use embedded_hal::digital::v2::{InputPin, OutputPin};
use cortex_m::delay::Delay;

pub struct DistanceSensor <T: OutputPin, E: InputPin> {
    trig: T,
    echo: E,
    timeout: u16,
}

impl<T: OutputPin, E: InputPin> DistanceSensor<T, E> {
    pub fn new(trig: T, echo: E, timeout: u16) -> Self {
        DistanceSensor { trig: trig, echo: echo, timeout: timeout }
    }
    
    pub fn get_value(&mut self, delay: &mut Delay) -> u16 {
        let mut wait_us: u16 = 0;
        let mut pulse_us: u16 = 0;
        
        let _ = self.trig.set_high();
        delay.delay_us(100);
        let _ = self.trig.set_low();
        
        loop {
            match self.echo.is_low().ok() {
                Some(true) => {
                    delay.delay_us(1);
                    wait_us = wait_us.saturating_add(1);
                    if wait_us >= self.timeout { break; }
                }
                Some(false) => { break; },
                None => { break; },
            }
        }
        
        loop {
            match self.echo.is_high().ok() {
                Some(true) => {
                    delay.delay_us(1);
                    pulse_us = pulse_us.saturating_add(1);
                    if pulse_us >= self.timeout { break; }
                }
                Some(false) => { break; },
                None => { break; },
            }
        }
        pulse_us
    }
}
