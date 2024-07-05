// build directions:
// cargo build --release
// elf2uf2-rs ./target/thumbv6m-none-eabi/release/usb-leds usb-leds.uf2

#![no_std]
#![no_main]

use rp_pico::entry;
use panic_halt as _;
use rp_pico::hal;
use rp_pico::hal::pac;
use rp_pico::hal::prelude::*;
use rp_pico::hal::gpio;
use rp_pico::hal::spi;
use rp_pico::hal::Timer;
//~ use embedded_hal::digital::v2::OutputPin;
use embedded_hal::prelude::_embedded_hal_blocking_spi_Write;
use embedded_time::rate::*;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

mod colors;
use crate::colors::{Pixel, PixelHsv};
mod leds;
use crate::leds::{Leds, LedType};
mod prng;
use crate::prng::Prng;
mod sprites;
use crate::sprites::RandomSprites;

const SERIAL_NUM: &str = "unique_name";
const LED_TYPE: LedType = LedType::Apa102;
//~ const LED_TYPE: LedType = LedType::Ws2801;
const NUM_LEDS: usize = 32;

//~ --APA102 LEDs
const BUF_SIZE: usize = 4 + (NUM_LEDS * 4) + ((NUM_LEDS + 1) / 2);
//~ --WS2801 LEDs
//~ const BUF_SIZE: usize = NUM_LEDS * 3;

#[derive(PartialEq)]
struct ColorMessage {
    pattern: u8,
    color_1: PixelHsv,
    color_2: PixelHsv,
}

impl ColorMessage {
    fn new() -> Self {
        Self { pattern: 0, color_1: PixelHsv::new(0.0, 0.0, 0.0), color_2: PixelHsv::new(0.0, 0.0, 0.0) } 
    }
    
    fn set_pattern(&mut self, pattern: u8) {
        self.pattern = pattern;
    }
    
    fn set_colors(&mut self, color_1: PixelHsv, color_2: PixelHsv) {
        self.color_1 = color_1;
        self.color_2 = color_2;
    }
}

fn bytes_to_pixelhsv(data: &[u8]) -> PixelHsv {
    let mut out = [0.0, 0.0, 0.0];
    for (index, value) in data.chunks(4).enumerate() {
        if index > 2 { break }
        out[index] = f32::from_le_bytes([value[0], value[1], value[2], value[3]]);
    }
    PixelHsv::new(out[0], out[1], out[2])
}

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

    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    
    //~ let mut led_pin = pins.led.into_push_pull_output();
    
    //~ let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

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
        .product("USB LEDs")
        .serial_number(SERIAL_NUM)
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();
    
    let _spi_sclk = pins.gpio2.into_mode::<gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio3.into_mode::<gpio::FunctionSpi>();
    //~ let _spi_miso = pins.gpio4.into_mode::<gpio::FunctionSpi>();

    let spi = spi::Spi::<_, _, 8>::new(pac.SPI0);
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    
    let mut rng = Prng::new(123456789);
    let mut sprite = RandomSprites::<NUM_LEDS>::new(1.0, 0.00001, &mut rng);
        
    let mut led = Leds::<NUM_LEDS, BUF_SIZE>::new(LED_TYPE);
    led.all_off();
    let _ = spi.write(&led.get_buffer()[..]);
    let mut last_update: u64 = 0;

    let mut current_message = ColorMessage::new();

    loop {
        // Check for new data
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
                        //~ --command 0: off
                        0 => {},
                        // --command 1: set pattern and color
                        1 => {
                            let mut message = ColorMessage::new();
                            message.set_pattern(usb_buf[1]);
                            
                            let color_1 = bytes_to_pixelhsv(&usb_buf[2..14]);
                            let color_2 = bytes_to_pixelhsv(&usb_buf[14..26]);
                            
                            message.set_colors(color_1, color_2);
                            if message != current_message { current_message = message }
                            
                            // --write "OK\n" to serial:
                            serial.write(&[0x4F, 0x4B, 0x0A]).ok();
                        },
                        //~ --command 2: get pattern and colors as binary data
                        2 => {
                            let start = timer.get_counter();
                            loop {
                                if serial.rts() { 
                                    break;
                                }
                                if timer.get_counter() - start >= 1000000 {
                                    break;
                                }
                            }
                            
                            //~ while !serial.rts() {}
                            
                            let color_1_h = current_message.color_1.get_h().to_le_bytes();
                            let color_1_s = current_message.color_1.get_s().to_le_bytes();
                            let color_1_v = current_message.color_1.get_v().to_le_bytes();
                            let color_2_h = current_message.color_2.get_h().to_le_bytes();
                            let color_2_s = current_message.color_2.get_s().to_le_bytes();
                            let color_2_v = current_message.color_2.get_v().to_le_bytes();
                            serial.write(&[current_message.pattern]).ok();
                            serial.write(&color_1_h[..]).ok();
                            serial.write(&color_1_s[..]).ok();
                            serial.write(&color_1_v[..]).ok();
                            serial.write(&color_2_h[..]).ok();
                            serial.write(&color_2_s[..]).ok();
                            serial.write(&color_2_v[..]).ok();
                            serial.write(&[0x0A]).ok();
                            
                            match serial.flush() {
                                Ok(_) => {},
                                Err(_) => {},
                            }
                        }
                        _ => {},
                    }
                }
            }
        }
        
        if timer.get_counter() - last_update >= 10000 {
            last_update = timer.get_counter();
            
            match current_message.pattern {
                0 => {
                    led.all_off();
                    spi.write(&led.get_buffer()[..]).ok();
                }
                1 => {
                    led.fill_gradient(&current_message.color_1, &current_message.color_2);
                    spi.write(&led.get_buffer()[..]).ok();
                }
                2 => {
                    led.fill_gradient_dual(&current_message.color_1, &current_message.color_2);
                    spi.write(&led.get_buffer()[..]).ok();
                }
                3 => {
                    led.fill_random(&current_message.color_1, &current_message.color_2, &sprite);
                    spi.write(&led.get_buffer()[..]).ok();
                }
                _ => {},
            }
            sprite.run(&mut rng);
        }
    }
}
