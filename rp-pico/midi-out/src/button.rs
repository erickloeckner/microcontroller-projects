use rp2040_hal::gpio::PinId;
use rp2040_hal::gpio::{Input, Pin, PullUp};
use embedded_hal::digital::v2::InputPin;
//~ use atsamd_hal::prelude::_atsamd_hal_embedded_hal_digital_v2_InputPin;

pub struct Button<P: PinId> {
    pin: Pin<P, Input<PullUp>>,
    debounce_time: u32,
    debounce_max: u32,
    debounce_state: DebounceState,
    state: bool,
    last_state: bool,
    toggle_state: bool,
    last_on: u32,
    last_off: u32,
}

impl<P: PinId> Button<P> {
    pub fn new(pin: Pin<P, Input<PullUp>>, debounce_time: u32) -> Self {
        Button {
            pin: pin,
            debounce_time: debounce_time,
            debounce_max: 20,
            debounce_state: DebounceState::Off,
            state: false,
            last_state: false,
            toggle_state: false,
            last_on: 0,
            last_off: 0,
        }
    }
    
    pub fn poll(&mut self, millis: u32) {
        let current_state = self.pin.is_low().unwrap();
        let last_on_delta = sub_handle_overflow(millis, self.last_on);
        let last_off_delta = sub_handle_overflow(millis, self.last_off);
        self.last_state = self.state;
        
        match self.debounce_state {
            DebounceState::Off => {
                //~ rising edge, change state to DebouncingOn and record last_on time
                if current_state == true {
                    self.debounce_state = DebounceState::DebouncingOn;
                    self.last_on = millis;
                }
            }
            DebounceState::DebouncingOff => {
                //~ pin state has stayed the same for the debounce_time, change debounce_state to Off
                if current_state == false && last_off_delta > self.debounce_time {
                    self.debounce_state = DebounceState::Off;
                    self.state = false;
                }
                //~ timeout, change state back to On
                if current_state == true && last_off_delta > self.debounce_max {
                    self.debounce_state = DebounceState::On;
                }
            }
            DebounceState::DebouncingOn => {
                //~ pin state has stayed the same for the debounce_time, change debounce_state to On
                if current_state == true && last_on_delta > self.debounce_time {
                    self.debounce_state = DebounceState::On;
                    self.state = true;
                    self.toggle_state = !self.toggle_state;
                }
                //~ timeout, change state back to Off
                if current_state == false && last_on_delta > self.debounce_max {
                    self.debounce_state = DebounceState::Off;
                }
            }
            DebounceState::On => {
                //~ falling edge, change state to DebouncingOff and record last_off time
                if current_state == false {
                    self.debounce_state = DebounceState::DebouncingOff;
                    self.last_off = millis;
                }
            }
        }
    }
    
    pub fn state(&self) -> bool {
        self.state
    }
    
    pub fn last_state(&self) -> bool {
        self.last_state
    }
    
    pub fn toggle_state(&self) -> bool {
        self.toggle_state
    }
}

fn sub_handle_overflow(millis: u32, last_time: u32) -> u32 {
    //~ until the millis variable overflows, it should always be larger than or equal to last_time
    if millis >= last_time {
        millis - last_time
    //~ once millis overflows and wraps around to zero, subtract last_time from the max u32 value
    //~ and add it to millis to prevent a panic
    } else {
        (u32::MAX - last_time) + millis
    }
}

enum DebounceState {
    Off,
    DebouncingOff,
    DebouncingOn,
    On,
}
