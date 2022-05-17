//! # Rainbow Example for the Adafruit KB2040
//!
//! Runs a rainbow-effect colour wheel on the on-board LED.
//!
//! Uses the `ws2812_pio` driver to control the LED, which in turns uses the
//! RP2040's PIO block.

#![no_std]
#![no_main]

mod keymatrix;

use defmt_rtt as _;
use panic_halt as _;
use rtic::app;

#[app(device = adafruit_kb2040::hal::pac, peripherals = true)]
mod app {

    use crate::keymatrix::*;

    use adafruit_kb2040::{
        hal::{self, resets, watchdog},
        pac, XOSC_CRYSTAL_FREQ,
    };
    use usb_device::{class_prelude::*, prelude::*};
    use usbd_hid::{
        descriptor::{generator_prelude::*, KeyboardReport},
        hid_class::HIDClass,
    };
    use usbd_serial::SerialPort;

    #[shared]
    struct Shared {
        usb_serial: SerialPort<'static, hal::usb::UsbBus>,
        usb_hid: usbd_hid::hid_class::HIDClass<'static, hal::usb::UsbBus>,
        usb_dev: usb_device::device::UsbDevice<'static, hal::usb::UsbBus>,
    }

    #[local]
    struct Local {}

    #[init(local = [usb_bus: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        //
        // initialise clocks
        //

        let mut resets = c.device.RESETS;
        let mut watchdog = hal::watchdog::Watchdog::new(c.device.WATCHDOG);

        // configure clock - default 125 MHz
        let clocks = hal::clocks::init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // setup USB

        let usb_bus = c
            .local
            .usb_bus
            .insert(UsbBusAllocator::new(hal::usb::UsbBus::new(
                c.device.USBCTRL_REGS,
                c.device.USBCTRL_DPRAM,
                clocks.usb_clock,
                true,
                &mut resets,
            )));

        let usb_serial = SerialPort::new(usb_bus);

        let usb_hid = HIDClass::new(usb_bus, KeyboardReport::desc(), 10);

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid((0x16c0), (0x27dd)))
            .manufacturer("ifacodes")
            .product("keebifa Keyboard")
            .serial_number("ifapersonal")
            .device_class(0x02)
            .build();

        (
            Shared {
                usb_serial,
                usb_hid,
                usb_dev,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [usb_serial, usb_hid, usb_dev])]
    fn usb_rx(cx: usb_rx::Context) {
        let usb_serial = cx.shared.usb_serial;
        let usb_hid = cx.shared.usb_hid;
        let usb_dev = cx.shared.usb_dev;

        (usb_serial, usb_hid, usb_dev).lock(|usb_serial, usb_hid, usb_dev| {
            //if usb_dev.poll(&mut [usb_hid]) {}

            if usb_dev.poll(&mut [usb_serial]) {
                let mut buf = [0u8; 64];
                match usb_serial.read(&mut buf) {
                    Err(_e) => {
                        // Do nothing
                        // let _ = serial_a.write(b"Error.");
                        // let _ = serial_a.flush();
                    }
                    Ok(0) => {
                        // Do nothing
                        let _ = usb_serial.write(b"Didn't received data.");
                        let _ = usb_serial.flush();
                    }
                    Ok(count) => {
                        buf.iter_mut().take(count).for_each(|b| {
                            b.make_ascii_uppercase();
                        });

                        // Send back to the host
                        let mut wr_ptr = &buf[..count];
                        while !wr_ptr.is_empty() {
                            let _ = usb_serial.write(wr_ptr).map(|len| {
                                wr_ptr = &wr_ptr[len..];
                            });
                        }
                    }
                }
            }
        });
    }
}
