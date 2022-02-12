#![no_std]
#![no_main]

pub mod button;
pub mod midi;

use core::cell::Cell;

use cortex_m_rt::entry;

use cortex_m::interrupt as cortex_interrupt;
use cortex_m::interrupt::Mutex;

use embedded_hal::digital::v2::OutputPin;

use embedded_time::duration::Extensions;
use embedded_time::rate::Baud;

use panic_halt as _;

use rp_pico::hal::pac;
use rp_pico::hal;
use rp_pico::hal::pac::interrupt;

use rp2040_hal::gpio::FunctionUart;
use rp2040_hal::timer::{Alarm0, Timer};
use rp2040_hal::uart::{DataBits, StopBits, UartConfig};

// global millisecond timer variable
static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

// the timer and alarm0 must be accessed in the interrupt function,
// so they need to be global static variables
static mut MILLIS_TIMER: Option<Timer> = None;
static mut MILLIS_ALARM: Option<Alarm0> = None;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
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
    
    // set up the main Timer object and Alarm0.
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut alarm = timer.alarm_0().unwrap();
    // enable the Alarm0 interrupt (TIMER_IRQ_0) and schedule for 1ms.
    alarm.enable_interrupt(&mut timer);
    let _ = alarm.schedule(1000_u32.microseconds());
    
    // move the timer and alarm objects into the global static variables.
    // this is safe to do before interrupts are enabled.
    unsafe {
        MILLIS_TIMER = Some(timer);
        MILLIS_ALARM = Some(alarm);
    }
    
    // enable the TIMER_IRQ_0 interrupt, which fires when alarm0 goes off.
    unsafe {
        pac::NVIC::unmask(hal::pac::Interrupt::TIMER_IRQ_0);
    };

    let sio = hal::Sio::new(pac.SIO);

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_config = UartConfig {
        baudrate: Baud(31250),
        data_bits: DataBits::Eight,
        stop_bits: StopBits::One,
        parity: None,
    };
    
    let mut uart = hal::uart::UartPeripheral::new(pac.UART0, &mut pac.RESETS)
        .enable(uart_config, clocks.peripheral_clock.into())
        .unwrap();
    
    let _tx_pin = pins.gpio0.into_mode::<FunctionUart>();
    let _rx_pin = pins.gpio1.into_mode::<FunctionUart>();

    let mut led_pin = pins.led.into_push_pull_output();
    
    let mut button1 = button::Button::new(pins.gpio2.into_pull_up_input(), 1);
    
    // the local millisecond counter variable
    let mut millis: u32 = 0;

    loop {
        // copy the global millisecond value into the local variable.
        // that allows the rest of the code in the main loop to run
        // without interrupts being disabled.
        cortex_interrupt::free(|cs| {
            millis = MILLIS.borrow(cs).get();
        });
        
        button1.poll(millis);
        if button1.state() == true {
            if button1.last_state() == false {
                midi::note_on(&mut uart, 1, 35, 63);
                led_pin.set_high().unwrap();
            }
        } else {
            if button1.last_state() == true {
                midi::note_on(&mut uart, 1, 35, 0);
                led_pin.set_low().unwrap();
            }
        }
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn TIMER_IRQ_0() {
    let timer_ref = MILLIS_TIMER.as_mut().unwrap();
    let alarm_ref = MILLIS_ALARM.as_mut().unwrap();
    // the alarm must be rescheduled after it goes off and the interrupt flag also
    // needs to be cleared to run continuously
    let _ = alarm_ref.schedule(1000_u32.microseconds());
    alarm_ref.clear_interrupt(timer_ref);
    
    // increment the global millisecond timer
    cortex_interrupt::free(|cs| MILLIS.borrow(cs).set(MILLIS.borrow(cs).get().wrapping_add(1)));
}
