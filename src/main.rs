//! # Rainbow Example for the Adafruit KB2040
//!
//! Runs a rainbow-effect colour wheel on the on-board LED.
//!
//! Uses the `ws2812_pio` driver to control the LED, which in turns uses the
//! RP2040's PIO block.

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
        hal::{
            self,
            clocks::Clock,
            gpio::{bank0::Gpio17, DynPin},
            pio::{PIOExt, SM0},
            Timer,
        },
        pac::PIO0,
        XOSC_CRYSTAL_FREQ,
    };
    use cortex_m::prelude::{
        _embedded_hal_watchdog_Watchdog, _embedded_hal_watchdog_WatchdogEnable,
    };
    use embedded_time::duration::Extensions;
    use smart_leds::{brightness, colors::*, SmartLedsWrite};
    use usb_device::{class_prelude::*, prelude::*};
    use ws2812_pio::Ws2812Direct;

    use keyberon::{
        debounce::Debouncer,
        key_code::{KbHidReport, KeyCode},
        layout::{Event, Layout},
        matrix::Matrix,
    };

    const COL_NUM: usize = 2;
    const ROW_NUM: usize = 2;

    #[shared]
    struct Shared {
        usb_hid:
            keyberon::hid::HidClass<'static, hal::usb::UsbBus, keyberon::keyboard::Keyboard<()>>,
        usb_dev: usb_device::device::UsbDevice<'static, hal::usb::UsbBus>,
        timer: hal::timer::Timer,
        alarm: hal::timer::Alarm0,
        ws: Ws2812Direct<PIO0, SM0, Gpio17>,
        #[lock_free]
        matrix: Matrix<DynPin, DynPin, COL_NUM, ROW_NUM>,
        #[lock_free]
        debouncer: Debouncer<[[bool; COL_NUM]; ROW_NUM]>,
        #[lock_free]
        layout: Layout<COL_NUM, ROW_NUM, 1>,
        #[lock_free]
        watchdog: hal::watchdog::Watchdog,
    }

    #[local]
    struct Local {
        led: u8,
    }

    #[init(local = [usb_bus: Option<usb_device::bus::UsbBusAllocator<hal::usb::UsbBus>> = None])]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        //
        // initialise clocks
        //

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
            .device_class(0x03)
            .build();

        //*******
        // Initalize pins and keyboard matrix.
        let sio = hal::Sio::new(c.device.SIO);

        let pins = adafruit_kb2040::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        let (mut pio, sm0, _, _, _) = c.device.PIO0.split(&mut resets);

        let mut ws = Ws2812Direct::new(
            pins.neopixel.into_mode(),
            &mut pio,
            sm0,
            clocks.peripheral_clock.freq(),
        );

        //ws.write([GREEN].iter().copied()).unwrap();

        let matrix: Matrix<DynPin, DynPin, COL_NUM, ROW_NUM> =
            cortex_m::interrupt::free(move |_cs| {
                Matrix::new(
                    [
                        pins.d2.into_pull_up_input().into(),
                        pins.d3.into_pull_up_input().into(),
                    ],
                    [
                        pins.a2.into_push_pull_output().into(),
                        pins.a1.into_push_pull_output().into(),
                    ],
                )
            })
            .unwrap();

        let debouncer =
            Debouncer::new([[false; COL_NUM]; ROW_NUM], [[false; COL_NUM]; ROW_NUM], 30);

        let layout = Layout::new(&TEST_LAYER);

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
                ws,
            },
            Local { led: 0 },
            init::Monotonics(),
        )
    }

    // #[idle(shared = [usb_hid, usb_dev, ws])]
    // fn idle(mut cx: idle::Context) -> ! {
    //     let usb_hid = cx.shared.usb_hid;
    //     let usb_dev = cx.shared.usb_dev;
    //     let ws = cx.shared.ws;
    //     (usb_hid, usb_dev, ws).lock(|h, d, w| loop {
    //         w.write([BLUE].iter().copied()).unwrap();
    //         if d.poll(&mut [h]) {
    //             h.poll();
    //         }
    //     })
    // }
    #[task(binds = TIMER_IRQ_0, priority = 1, shared = [usb_hid, timer, alarm, matrix, debouncer, layout, watchdog, ws], local = [led])]
    fn timer_irq(mut cx: timer_irq::Context) {
        // Clear Interrupt
        let mut alarm = cx.shared.alarm;
        alarm.lock(|a| {
            a.clear_interrupt();
            let _ = a.schedule(1000.microseconds());
        });

        cx.shared.watchdog.feed();

        (cx.shared.ws).lock(|w| {
            w.write(brightness([GREEN].iter().copied(), 32)).unwrap();
        });

        for event in cx.shared.debouncer.events(cx.shared.matrix.get().unwrap()) {
            cx.shared.layout.event(event);
        }
        cx.shared.layout.tick();

        let report: KbHidReport = cx.shared.layout.keycodes().collect();
        if !cx
            .shared
            .usb_hid
            .lock(|h| h.device_mut().set_keyboard_report(report.clone()))
        {
            return;
        }
        while let Ok(0) = cx.shared.usb_hid.lock(|h| h.write(report.as_bytes())) {}
    }
    // #[task(priority = 2, capacity = 8)]
    // fn report(mut cx: report::Context) {
    //     let report = cx.local.report;
    //     let key_table = cx.local.key_table;
    //     let result = cx.local.matrix.poll().unwrap();
    //     for (y, row) in result.iter().enumerate() {
    //         for (x, _) in row.iter().enumerate() {
    //             update_report(report, key_table[y][x], result[y][x])
    //         }
    //     }
    //     cx.shared.usb_hid.lock(|s| s.push_input(report).unwrap());
    // }

    #[task(binds = USBCTRL_IRQ, priority = 3, shared = [usb_hid, usb_dev, ws])]
    fn usb_rx(cx: usb_rx::Context) {
        //let usb_serial = cx.shared.usb_serial;
        let usb_hid = cx.shared.usb_hid;
        let usb_dev = cx.shared.usb_dev;
        let ws = cx.shared.ws;

        (usb_hid, usb_dev, ws).lock(|h, d, w| {
            w.write(brightness([RED].iter().copied(), 32)).unwrap();
            if d.poll(&mut [h]) {
                h.poll();
            }
        });
    }
}
