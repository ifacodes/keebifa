//! # keebifa Keyboard Firmware for Adafruit KB2040
//!
//! Keyboard firmware for a Alice layout keyboard written using Keyberon
//!

#![no_std]
#![no_main]

mod layout;
use defmt_rtt as _;
use panic_halt as _;
use rtic::app;

#[app(device = adafruit_kb2040::hal::pac, peripherals = true, dispatchers = [PIO0_IRQ_0])]
mod app {

    use crate::layout::*;

    use adafruit_kb2040::{
        hal::{self, gpio::DynPin, Timer},
        XOSC_CRYSTAL_FREQ,
    };
    use cortex_m::prelude::{
        _embedded_hal_watchdog_Watchdog, _embedded_hal_watchdog_WatchdogEnable,
    };
    use embedded_time::duration::Extensions;
    use usb_device::{class_prelude::*, prelude::*};

    use keyberon::{debounce::Debouncer, key_code::KbHidReport, layout::Layout, matrix::Matrix};

    const COL_NUM: usize = 13;
    const ROW_NUM: usize = 5;

    #[shared]
    struct Shared {
        usb_hid:
            keyberon::hid::HidClass<'static, hal::usb::UsbBus, keyberon::keyboard::Keyboard<()>>,
        usb_dev: usb_device::device::UsbDevice<'static, hal::usb::UsbBus>,
        timer: hal::timer::Timer,
        alarm: hal::timer::Alarm0,
        #[lock_free]
        matrix: Matrix<DynPin, DynPin, COL_NUM, ROW_NUM>,
        #[lock_free]
        debouncer: Debouncer<[[bool; COL_NUM]; ROW_NUM]>,
        #[lock_free]
        layout: Layout<COL_NUM, ROW_NUM, 2>,
        #[lock_free]
        watchdog: hal::watchdog::Watchdog,
    }

    #[local]
    struct Local {}

    #[init(local = [usb_bus: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        //
        // initialise clocks

        let mut resets = c.device.RESETS;
        let mut watchdog = hal::watchdog::Watchdog::new(c.device.WATCHDOG);
        watchdog.pause_on_debug(false);

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

        let usb_hid = keyberon::new_class(usb_bus, ());

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("ifacodes")
            .product("keebifa Keyboard")
            .serial_number("ifapersonal")
            .device_class(0x02)
            .build();

        // Initalize pins and keyboard matrix.

        let sio = hal::Sio::new(c.device.SIO);

        let pins = adafruit_kb2040::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // initialize keyboard related structs

        let matrix: Matrix<DynPin, DynPin, COL_NUM, ROW_NUM> =
            cortex_m::interrupt::free(move |_cs| {
                Matrix::new(
                    [
                        pins.tx.into_pull_up_input().into(),
                        pins.rx.into_pull_up_input().into(),
                        pins.d2.into_pull_up_input().into(),
                        pins.d3.into_pull_up_input().into(),
                        pins.d4.into_pull_up_input().into(),
                        pins.d5.into_pull_up_input().into(),
                        pins.d6.into_pull_up_input().into(),
                        pins.d7.into_pull_up_input().into(),
                        pins.d8.into_pull_up_input().into(),
                        pins.d9.into_pull_up_input().into(),
                        pins.d10.into_pull_up_input().into(),
                        pins.mosi.into_pull_up_input().into(),
                        pins.miso.into_pull_up_input().into(),
                    ],
                    [
                        pins.a3.into_push_pull_output().into(),
                        pins.a2.into_push_pull_output().into(),
                        pins.a1.into_push_pull_output().into(),
                        pins.a0.into_push_pull_output().into(),
                        pins.sclk.into_push_pull_output().into(),
                    ],
                )
            })
            .unwrap();

        let debouncer =
            Debouncer::new([[false; COL_NUM]; ROW_NUM], [[false; COL_NUM]; ROW_NUM], 30);

        let layout = Layout::new(&ALICE_LAYOUT);

        // initalisze timer, alarm, and watchdog

        let mut timer = Timer::new(c.device.TIMER, &mut resets);
        let mut alarm = timer.alarm_0().unwrap();
        let _ = alarm.schedule(1000.microseconds());
        alarm.enable_interrupt();

        watchdog.start(10_000.microseconds());

        (
            Shared {
                usb_hid,
                usb_dev,
                timer,
                alarm,
                matrix,
                debouncer,
                layout,
                watchdog,
            },
            Local {},
            init::Monotonics(),
        )
    }

    #[task(binds = TIMER_IRQ_0, priority = 1, shared = [usb_hid, timer, alarm, matrix, debouncer, layout, watchdog])]
    fn timer_irq(mut cx: timer_irq::Context) {
        // Clear Interrupt
        let mut alarm = cx.shared.alarm;
        alarm.lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(1000.microseconds());
        });

        cx.shared.watchdog.feed();

        for event in cx.shared.debouncer.events(cx.shared.matrix.get().unwrap()) {
            cx.shared.layout.event(event);
        }
        cx.shared.layout.tick();

        let report: KbHidReport = cx.shared.layout.keycodes().collect();
        if cx
            .shared
            .usb_hid
            .lock(|h| h.device_mut().set_keyboard_report(report.clone()))
        {
            while let Ok(0) = cx.shared.usb_hid.lock(|h| h.write(report.as_bytes())) {}
        }
    }

    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [usb_hid, usb_dev])]
    fn usb_rx(cx: usb_rx::Context) {
        let usb_hid = cx.shared.usb_hid;
        let usb_dev = cx.shared.usb_dev;

        (usb_hid, usb_dev).lock(|h, d| {
            if d.poll(&mut [h]) {
                h.poll();
            }
        });
    }
}
