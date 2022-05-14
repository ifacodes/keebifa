//! # Rainbow Example for the Adafruit KB2040
//!
//! Runs a rainbow-effect colour wheel on the on-board LED.
//!
//! Uses the `ws2812_pio` driver to control the LED, which in turns uses the
//! RP2040's PIO block.

#![no_std]
#![no_main]

use core::iter::once;
use cortex_m_rt::entry;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::{Extensions, Microseconds};
use panic_halt as _;

use adafruit_kb2040::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        pac,
        pio::PIOExt,
        timer::Timer,
        watchdog::Watchdog,
        Sio,
    },
    XOSC_CRYSTAL_FREQ,
};
use smart_leds::{
    brightness,
    colors::{BLACK, INDIGO},
    SmartLedsWrite, RGB8,
};
use ws2812_pio::Ws2812;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this
/// function as soon as all global variables are initialised.
///
/// The function configures the RP2040 peripherals, then the LED, then runs
/// the colour wheel in an infinite loop.
#[entry]
fn main() -> ! {
    // Configure the RP2040 peripherals

    let mut pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);

    let pins = adafruit_kb2040::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut delay = timer.count_down();

    // Configure the addressable LED
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);

    let mut ws = Ws2812::new(
        pins.neopixel.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    // Infinite colour wheel loop

    let mut output = pins.a0.into_push_pull_output();
    let input = pins.a1.into_pull_down_input();

    output.set_high().unwrap();

    let mut n: u8 = 128;
    loop {
        if input.is_low().unwrap() {
            ws.write(brightness(once(wheel(n)), 32)).unwrap();
            n = n.wrapping_add(1);
        } else {
            ws.write(brightness(once(BLACK), 32)).unwrap();
        }
        delay.start(25.milliseconds());
        let _ = nb::block!(delay.wait());
    }
}

fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        // No green in this sector - red and blue only
        (255 - (wheel_pos * 3), 0, wheel_pos * 3).into()
    } else if wheel_pos < 170 {
        // No red in this sector - green and blue only
        wheel_pos -= 85;
        (0, wheel_pos * 3, 255 - (wheel_pos * 3)).into()
    } else {
        // No blue in this sector - red and green only
        wheel_pos -= 170;
        (wheel_pos * 3, 255 - (wheel_pos * 3), 0).into()
    }
}
// Blinks the LED on a Pico board
//
// This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
// #![no_std]
// #![no_main]

// use cortex_m_rt::entry;
// use defmt::*;
// use defmt_rtt as _;
// use embedded_hal::digital::v2::OutputPin;
// use embedded_time::fixed_point::FixedPoint;
// use panic_probe as _;

// // Provide an alias for our BSP so we can switch targets quickly.
// // Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
// use rp_pico as bsp;
// // use sparkfun_pro_micro_rp2040 as bsp;

// use bsp::hal::{
//     clocks::{init_clocks_and_plls, Clock},
//     pac,
//     sio::Sio,
//     watchdog::Watchdog,
// };

// #[entry]
// fn main() -> ! {
//     info!("Program start");
//     let mut pac = pac::Peripherals::take().unwrap();
//     let core = pac::CorePeripherals::take().unwrap();
//     let mut watchdog = Watchdog::new(pac.WATCHDOG);
//     let sio = Sio::new(pac.SIO);

//     // External high-speed crystal on the pico board is 12Mhz
//     let external_xtal_freq_hz = 12_000_000u32;
//     let clocks = init_clocks_and_plls(
//         external_xtal_freq_hz,
//         pac.XOSC,
//         pac.CLOCKS,
//         pac.PLL_SYS,
//         pac.PLL_USB,
//         &mut pac.RESETS,
//         &mut watchdog,
//     )
//     .ok()
//     .unwrap();

//     let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

//     let pins = bsp::Pins::new(
//         pac.IO_BANK0,
//         pac.PADS_BANK0,
//         sio.gpio_bank0,
//         &mut pac.RESETS,
//     );

//     let mut led_pin = pins.led.into_push_pull_output();

//     loop {
//         info!("on!");
//         led_pin.set_high().unwrap();
//         delay.delay_ms(500);
//         info!("off!");
//         led_pin.set_low().unwrap();
//         delay.delay_ms(500);
//     }
// }

// // End of file
