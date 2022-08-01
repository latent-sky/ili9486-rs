//! Displays something on a display with a ILI9486 controllers and an 8080 style parallel port interface
//! connected to a Raspberry Pi Pico  board.
//!
//! Useful examample code is https://github.com/adoble/ili9486-driver/blob/master/examples/hello_world.rs
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_time::fixed_point::FixedPoint;
use panic_probe as _;

// Using the Raspberry Pi Pico board
use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use display_interface_parallel_gpio::{Generic8BitBus, PGPIO8BitInterface};

use ili9486::{PixelFormat, ILI9486};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};

#[entry]
fn main() -> ! {
    let mut dp = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(dp.WATCHDOG);
    let sio = Sio::new(dp.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        dp.XOSC,
        dp.CLOCKS,
        dp.PLL_SYS,
        dp.PLL_USB,
        &mut dp.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Required for the display initialisation
    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(dp.IO_BANK0, dp.PADS_BANK0, sio.gpio_bank0, &mut dp.RESETS);

    // Set up the pins used to drive the display over the 8080 parallel bus
    let d0 = pins.gpio0.into_push_pull_output();
    let d1 = pins.gpio1.into_push_pull_output();
    let d2 = pins.gpio2.into_push_pull_output();
    let d3 = pins.gpio3.into_push_pull_output();
    let d4 = pins.gpio4.into_push_pull_output();
    let d5 = pins.gpio5.into_push_pull_output();
    let d6 = pins.gpio6.into_push_pull_output();
    let d7 = pins.gpio7.into_push_pull_output();

    // The control pins
    let lcd_reset_pin = pins.gpio8.into_push_pull_output();
    let mut lcd_cs_pin = pins.gpio9.into_push_pull_output();
    let lcd_dcx_pin = pins.gpio10.into_push_pull_output();
    let lcd_write_pin = pins.gpio11.into_push_pull_output();

    // Note: The lcd_read pin is never used and is pulled high in the circuit.

    // Default conditions
    lcd_cs_pin.set_high().unwrap();

    // Set up the pico onboard lad as a diagnostic light
    let mut led_pin = pins.led.into_push_pull_output();

    let data_bus = Generic8BitBus::new((d0, d1, d2, d3, d4, d5, d6, d7))
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    // The display interface for a parallel GPIO 8080 type interface which implements the WriteOnlyDataCommand trait
    let display_interface = PGPIO8BitInterface::new(data_bus, lcd_dcx_pin, lcd_write_pin);

    let mut lcd_driver = ILI9486::new(display_interface, lcd_cs_pin, lcd_reset_pin)
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    lcd_driver
        .init(&mut delay)
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    lcd_driver
        .set_pixel_format(PixelFormat::Rgb565)
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    lcd_driver
        //.set_orientation(ili9486::Orientation::Portrait)
        .set_orientation(ili9486::Orientation::Landscape)
        .unwrap(); //TODO also sends MemoryAccessControl command

    lcd_driver
        .set_brightness(0xFF) // Lowest brightness
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    lcd_driver.set_window(0, 0, 480, 360).unwrap();

    // Clear the screen with a black background
    lcd_driver.fill(0x0000).unwrap();

    // Draw a line.
    (0..100)
        .map(|i| Pixel(Point::new(i, i * 2), Rgb565::BLACK))
        .draw(&mut lcd_driver)
        .unwrap_or_else(|_| error(&mut led_pin, &mut delay));

    // Create a rectangle
    let rect_style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::RED)
        .stroke_width(10)
        .fill_color(Rgb565::GREEN)
        .build();

    Rectangle::new(Point::new(30, 20), Size::new(100, 70))
        .into_styled(rect_style)
        .draw(&mut lcd_driver)
        //.unwrap_or_else(|_err| led_pin.set_high().unwrap());
        .unwrap();

    loop {
        // Continuously light the on board led if we have reached this far withour amy errors
        led_pin.set_high().unwrap();
    }
}

// ------------   Diagnostic functions. ---------------------

// Flash the led if an error has been detected.
#[allow(dead_code)]
fn error<P, D>(led_pin: &mut P, delay: &mut D) -> !
where
    P: embedded_hal::digital::v2::OutputPin,
    D: embedded_hal::blocking::delay::DelayMs<u32>,
{
    loop {
        let mut _result = led_pin.set_high();
        delay.delay_ms(250);
        _result = led_pin.set_low();
        delay.delay_ms(250);
        match _result {
            Ok(_) => (),
            Err(_) => (),
        }
    }
}

// Pulse pin up and down a bit with a 20 ms period so we know where we are
#[allow(dead_code)]
fn pulse<P, D>(pulsed_pin: &mut P, delay: &mut D) -> Result<(), ()>
where
    P: embedded_hal::digital::v2::OutputPin,
    D: embedded_hal::blocking::delay::DelayMs<u32>,
{
    for _i in 0..10 {
        pulsed_pin.set_low().map_err(|_| ())?;
        delay.delay_ms(10);
        pulsed_pin.set_high().map_err(|_| ())?;
        delay.delay_ms(10);
    }
    Ok(())
}
