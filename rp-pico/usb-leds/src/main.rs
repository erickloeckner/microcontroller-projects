// build directions:
// NAME=serial_num LED_COUNT=10 LED_TYPE=apa102 cargo build --release
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
use rp2040_hal::fugit::{Duration, ExtU64, RateExtU32};
use cortex_m::prelude::*;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

mod colors;
use crate::colors::PixelHsv;
mod leds;
use crate::leds::{BufferType, Leds, LedType};
mod prng;
use crate::prng::Prng;
mod sprites;
use crate::sprites::RandomSprites;

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
    
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let update_time: Duration<u64, 1, 1000000> = 10_u64.millis();
    let serial_timeout: Duration<u64, 1, 1000000> = 1_000_u64.millis();
    let mut last_update = timer.get_counter();

    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = SerialPort::new(&usb_bus);
    let serial_num = env!("NAME");
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Eric Kloeckner")
            .product("USB LEDs")
            .serial_number(serial_num)])
        .unwrap()
        .device_class(2)
        .build();
    
    let spi_sclk: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> = pins.gpio2.reconfigure();
    let spi_mosi: gpio::Pin<_, gpio::FunctionSpi, gpio::PullNone> = pins.gpio3.reconfigure();
    let spi_miso: gpio::Pin<_, gpio::FunctionSpi, gpio::PullUp> = pins.gpio4.reconfigure();

    let spi = spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        1_000_000u32.Hz(),
        embedded_hal::spi::MODE_0,
    );
    
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let pwm0 = &mut pwm_slices.pwm0;
    pwm0.set_ph_correct();
    pwm0.enable();
    
    let pwm1 = &mut pwm_slices.pwm1;
    pwm1.set_ph_correct();
    pwm1.enable();

    let r_channel = &mut pwm0.channel_a;
    r_channel.output_to(pins.gpio16);
    
    let g_channel = &mut pwm0.channel_b;
    g_channel.output_to(pins.gpio17);
    
    let b_channel = &mut pwm1.channel_a;
    b_channel.output_to(pins.gpio18);
    
    let led_count = match usize::from_str_radix(env!("LED_COUNT"), 10) {
        Ok(v) => v,
        Err(_) => 1,
    };

    let led_type = match env!("LED_TYPE") {
        "apa102" => LedType::Apa102,
        "ws2801" => LedType::Ws2801,
        "analog" => LedType::Analog,
        _ => LedType::Apa102,
    };

    let mut rng = Prng::new(123456789);
    let mut sprite = RandomSprites::new(led_count, 1.0, 0.00001, &mut rng);

    let mut phase: f32 = 0.0;
    let phase_step: f32 = 0.0001;

    let mut led = Leds::new(led_count, led_type);
    led.all_off();
    
    match led.buffer() {
        BufferType::Addressable(b) => {
            spi.write(&b[..led.buf_end()]).ok();
        }
        BufferType::Analog(b) => {
            r_channel.set_duty(b[0]);
            g_channel.set_duty(b[1]);
            b_channel.set_duty(b[2]);
        }
    }

    let mut current_message = ColorMessage::new();

    //let sine_test = hal::rom_data::float_funcs::fsin(0.5);

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
                                if timer.get_counter() - start >= serial_timeout {
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
        
        if timer.get_counter() - last_update >= update_time {
            last_update = timer.get_counter();
            
            match current_message.pattern {
                0 => {
                    led.all_off();
                }
                1 => {
                    led.fill_gradient(&current_message.color_1, &current_message.color_2);
                }
                2 => {
                    led.fill_gradient_dual(&current_message.color_1, &current_message.color_2);
                }
                3 => {
                    led.fill_triangle(&current_message.color_1, &current_message.color_2, phase);
                }
                4 => {
                    led.fill_random(&current_message.color_1, &current_message.color_2, &sprite);
                }
                _ => {},
            }
            
            match led.buffer() {
                BufferType::Addressable(b) => {
                    spi.write(&b[..led.len()]).ok();
                }
                BufferType::Analog(b) => {
                    r_channel.set_duty(b[0]);
                    g_channel.set_duty(b[1]);
                    b_channel.set_duty(b[2]);
                }
            }
            
            sprite.run(&mut rng);
            phase = (phase + phase_step) % 1.0;
        }
    }
}
