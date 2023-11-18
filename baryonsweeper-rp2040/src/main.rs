//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::{entry, hal::{self, gpio::bank0::{Gpio0, Gpio1}, uart::Parity}};
use defmt::*;
use defmt_rtt as _;

use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    Clock,
    clocks::init_clocks_and_plls,
    pac::{self, interrupt},
    sio::Sio,
    watchdog::Watchdog,
    Timer,
    uart::{self, UartConfig, DataBits, StopBits},
    fugit::{RateExtU32, ExtU64},
    usb::UsbBus,
};


// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

use baryonsweeper::BaryonSweeper;

/// The USB Device Driver (shared with the interrupt).
static mut USB_DEVICE: Option<UsbDevice<hal::usb::UsbBus>> = None;

/// The USB Bus Driver (shared with the interrupt).
static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;

/// The USB Serial Device Driver (shared with the interrupt).
static mut USB_SERIAL: Option<SerialPort<hal::usb::UsbBus>> = None;


#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

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
 
    unsafe {
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
        USB_SERIAL = Some(SerialPort::new(usb_bus)); 
        //USB_SERIAL.unwrap().write(b"Hello!\r\n").unwrap(); 
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

    let logger = embedded_logger::CombinedLogger::<UsbBus,256>::new(usb_serial);
    let mut baryon_sweeper = BaryonSweeper::new(uart, timer.count_down(), led_pin, 500.millis(), logger);

    baryon_sweeper.sweep();
    core::unreachable!()
}

#[allow(non_snake_case)]
#[interrupt]
unsafe fn USBCTRL_IRQ() {
    let usb_dev = unsafe { USB_DEVICE.as_mut().unwrap() };
    let usb_serial = unsafe { USB_SERIAL.as_mut().unwrap() };
    if usb_dev.poll(&mut [usb_serial]) {
        let mut buf = [0u8; 64];
        match usb_serial.read(&mut buf) {
            Err(_e) => {
                // Do nothing
            }
            Ok(0) => {
                // Do nothing
            }
            Ok(_count) => {
                // Do nothing
            }
        }
    }
}

