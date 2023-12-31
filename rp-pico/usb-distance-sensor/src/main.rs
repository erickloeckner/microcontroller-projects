// build directions:
// cargo build --release && elf2uf2-rs ./target/thumbv6m-none-eabi/release/usb-distance-sensor usb-distance-sensor.uf2

#![no_std]
#![no_main]

mod buffer;
use crate::buffer::RingBuffer;
mod sensor;
use crate::sensor::DistanceSensor;

use embedded_time::rate::*;

use panic_halt as _;

use rp_pico::hal::prelude::*;
use rp_pico::entry;
use rp_pico::hal::pac;
use rp_pico::hal;
use rp_pico::hal::Timer;

use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

use embedded_hal::digital::v2::OutputPin;

const BUFFER_SIZE: usize = 20;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Eric Kloeckner")
        .product("USB Distance Sensor")
        .serial_number("001")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    let mut sensor1 = DistanceSensor::new(
        pins.gpio20.into_push_pull_output(),
        pins.gpio21.into_pull_down_input(),
        65535u16,
    );
    let mut buf = RingBuffer::<u16, BUFFER_SIZE>::new();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut last_update: u64 = 0;
    
    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            let mut usb_buf = [0u8; 64];
            match serial.read(&mut usb_buf) {
                Err(_e) => {
                    // Do nothing
                }
                Ok(0) => {
                    // Do nothing
                }
                Ok(_count) => {
                    //~ --first byte is the command type
                    match usb_buf[0] {
                        //~ --command 0:
                        0 => {
                            
                        },
                        // --command 1: 
                        1 => {
                            /*
                            let start = timer.get_counter();
                            loop {
                                if serial.rts() { 
                                    break;
                                }
                                if timer.get_counter() - start >= 5000000 {
                                    //~ serial.write(&[0, 1, 2, 3).ok();
                                    led_pin.set_high().ok();
                                    break;
                                }
                            }
                            */
                            
                            let dist = buf.mean();
                            serial.write(&dist.to_le_bytes()).ok();
                            match serial.flush() {
                                Ok(_) => {},
                                Err(_) => {},
                            }
                        },
                        2 => {
                            //~ serial.write(&dist.to_le_bytes()).ok();
                        }
                        _ => {},
                    }
                }
            }
        }
        //~ buf.push(sensor1.get_value(&mut delay));
        //~ delay.delay_ms(10);
        
        if timer.get_counter() - last_update >= 10000 {
            last_update = timer.get_counter();
            buf.push(sensor1.get_value(&mut delay));
        }
    }
}
