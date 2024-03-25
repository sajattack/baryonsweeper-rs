#![no_std]
#![no_main]

use bsp::{entry, hal::{self, gpio::bank0::{Gpio0, Gpio1}, uart::Parity}};
use log::{LevelFilter, info};

use defmt_rtt as _;

//defmt::timestamp!("{=u32:us}", {
    // NOTE(interrupt-safe) single instruction volatile read operation
    //SYST::get_current()
//});

use panic_probe as _;
//use panic_halt as _;

use rp_pico as bsp;

use bsp::hal::{
    Clock,
    clocks::init_clocks_and_plls,
    pac::{self, interrupt, SYST},
    sio::Sio,
    watchdog::Watchdog,
    Timer,
    uart::{self, UartConfig, DataBits, StopBits},
    fugit::{RateExtU32, ExtU64},
    usb::UsbBus,
};

// USB Device support
//use usb_device::{class_prelude::*, prelude::*};

//use embedded_logger::UsbLogger;
//use embedded_logger::RTTLogger;

// USB Communications Class Device support
//use usbd_serial::SerialPort;

use baryonsweeper::BaryonSweeper;

/// The USB Device Driver (shared with the interrupt).
//static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
//static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Serial Device Driver (shared with the interrupt).
//static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

//static mut LOGGER: Option<UsbLogger::<UsbBus,256>> = None;

//const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
//static LOGGER: RTTLogger = RTTLogger::new(LOG_LEVEL);

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let mut core = pac::CorePeripherals::take().unwrap();
    core.SYST.enable_counter();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    info!("Program start");

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let led_pin = pins.led.into_push_pull_output();

    type UartPins = (
        hal::gpio::Pin<Gpio0, hal::gpio::FunctionUart, hal::gpio::PullNone>,
        hal::gpio::Pin<Gpio1, hal::gpio::FunctionUart, hal::gpio::PullNone>,
    );

    let uart_pins: UartPins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.reconfigure(),
        // UART RX (characters received by RP2040) on pin 2 (GPIO1)
        pins.gpio1.reconfigure(),
    );

    let uart = uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(19200.Hz(), DataBits::Eight, Some(Parity::Even), StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // Set up the USB driver
 
    /*unsafe {
        pac::NVIC::unmask(hal::pac::Interrupt::USBCTRL_IRQ);

        USB_BUS = Some(UsbBusAllocator::new(UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )))
    };

    let usb_bus = unsafe { USB_BUS.as_mut().unwrap() };


    // Set up the USB Communications Class Device driver
    unsafe { 
        USB_SERIAL = Some(SerialPort::new(usb_bus).into());
    }

    // Create a USB device with a fake VID and PID
    unsafe { USB_DEVICE = Some(UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("sajattack")
        .product("BaryonSweeper-rs")
        .serial_number("TEST")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build())
    };

    let usb_serial = unsafe { USB_SERIAL.as_mut().unwrap() };
    let logger = UsbLogger::<UsbBus,256>::new(usb_serial);

    unsafe { LOGGER = Some(logger) };
    unsafe { log::set_logger_racy( LOGGER.as_ref().unwrap() ).unwrap(); }
    */

    //rtt_init_print!();
    //log::set_logger(&LOGGER)
        //.map(|()| log::set_max_level(LOG_LEVEL))
        //.unwrap();

    let mut baryon_sweeper = BaryonSweeper::new(uart, timer.count_down(), led_pin, 500.millis());

    baryon_sweeper.sweep();
    core::unreachable!()
}

/*#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    unsafe {
        if let Some(ref mut usb_dev) = &mut USB_DEVICE {
            if let Some(ref mut usb_serial) = &mut USB_SERIAL {
                usb_dev.poll(&mut [usb_serial]);

                // Make the other side happy
                let mut buf = [0u8; 16];
                let _ = usb_serial.read(&mut buf);
            }
        }
    };
}*/

