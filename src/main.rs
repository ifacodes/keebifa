//! # Rainbow Example for the Adafruit KB2040
//!
//! Runs a rainbow-effect colour wheel on the on-board LED.
//!
//! Uses the `ws2812_pio` driver to control the LED, which in turns uses the
//! RP2040's PIO block.

#![no_std]
#![no_main]

mod keymatrix;

use keymatrix::*;

use adafruit_kb2040::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        gpio::{AnyPin, DynPin, FunctionConfig, ValidPinMode},
        pac::{self, interrupt},
        pio::{PIOExt, StateMachineIndex, SM0},
        timer::Timer,
        usb,
        watchdog::Watchdog,
        Sio,
    },
    pac::PIO0,
    XOSC_CRYSTAL_FREQ,
};
use core::iter::once;
use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::{Extensions, Microseconds};
use nb::block;
use panic_halt as _;
use smart_leds::{
    brightness,
    colors::{BLACK, INDIGO},
    SmartLedsWrite, RGB8,
};
use usb_device::{class_prelude::*, prelude::*};
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::hid_class::HIDClass;

use ws2812_pio::Ws2812;

static mut USB_DEVICE: Option<UsbDevice<usb::UsbBus>> = None;
static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBus>> = None;
static mut USB_HID: Option<HIDClass<usb::UsbBus>> = None;
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
    info!("Program Start");
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

    let usb_bus = UsbBusAllocator::new(usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    unsafe {
        USB_BUS = Some(usb_bus);
    }

    let bus_ref = unsafe { USB_BUS.as_ref().unwrap() };

    let usb_hid = HIDClass::new(bus_ref, KeyboardReport::desc(), 60);

    unsafe {
        USB_HID = Some(usb_hid);
    }

    let usb_dev = UsbDeviceBuilder::new(bus_ref, UsbVidPid(0x16c0, 0x27db))
        .manufacturer("ifacodes")
        .product("keebifa")
        .serial_number("1010")
        .device_class(0xEF)
        .build();

    unsafe {
        USB_DEVICE = Some(usb_dev);
    }

    unsafe {
        pac::NVIC::unmask(pac::interrupt::USBCTRL_IRQ);
    }

    let sio = Sio::new(pac.SIO);

    let pins = adafruit_kb2040::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    // let mut delay = timer.count_down();

    let mut mat: Matrix<DynPin, DynPin, 13, 5> = Matrix::new(
        [
            pins.tx.into(),
            pins.rx.into(),
            pins.d2.into(),
            pins.d3.into(),
            pins.d4.into(),
            pins.d5.into(),
            pins.d6.into(),
            pins.d7.into(),
            pins.d8.into(),
            pins.d9.into(),
            pins.d10.into(),
            pins.d11.into(),
            pins.mosi.into(),
        ],
        [
            pins.a0.into(),
            pins.a1.into(),
            pins.a2.into(),
            pins.a3.into(),
            pins.miso.into(),
        ],
    )
    .unwrap();

    loop {
        // let _ = mat.poll(&mut ws).unwrap();
        //let _ = serial.write(b"Hello World!\r\n");
        // delay.start(23.milliseconds());
        // let _ = nb::block!(delay.wait()).unwrap();
    }
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    // Handle USB request
    let usb_dev = USB_DEVICE.as_mut().unwrap();
    let usb_hid = USB_HID.as_mut().unwrap();
    usb_dev.poll(&mut [usb_hid]);
}
