#![no_std]
#![no_main]

use baryonsweeper::BaryonSweeper;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use itsybitsy_m0 as bsp;

use bsp::hal;
use bsp::pac;
use bsp::entry;

use hal::clock::GenericClockController;
use hal::prelude::*;
use hal::timer::TimerCounter;
use hal::usb::UsbBus;

use usb_device::class_prelude::*;

use pac::{CorePeripherals, Peripherals};
use pac::interrupt;
use pac::NVIC;

use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    let mut pm = peripherals.PM;
    let pins = bsp::Pins::new(peripherals.PORT);


    let gclk0 = clocks.gclk0();
    let tc45 = &clocks.tc4_tc5(&gclk0).unwrap();
    // instantiate a timer objec for the TC4 peripheral
    let timer = TimerCounter::tc4_(tc45, peripherals.TC4, &mut pm);


    // Take peripheral and pins
    let uart_sercom  = peripherals.SERCOM0;
    let uart_rx = pins.d0;
    let uart_tx = pins.d1;

    let uart = bsp::uart(
        &mut clocks,
        19200.hz(),
        uart_sercom,
        &mut pm,
        uart_rx,
        uart_tx,
    );

     let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(bsp::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut pm,
            pins.usb_dm,
            pins.usb_dp,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2222, 0x3333))
                .manufacturer("Fake company")
                .product("Serial port")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }

    let led_pin: bsp::RedLed = pins.d13.into();


    let usb_serial = unsafe { USB_SERIAL.as_mut().unwrap() };
    let logger = embedded_logger::CombinedLogger::<UsbBus,256>::new(usb_serial);

    let timeout: hal::time::Nanoseconds = 500.ms().into();
    let mut baryon_sweeper = BaryonSweeper::new(uart, timer, led_pin, timeout, logger);
    baryon_sweeper.sweep();
    core::unreachable!()


}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(ref mut usb_serial) = &mut USB_SERIAL {
                usb_dev.poll(&mut [usb_serial]);

                // Make the other side happy
                let mut buf = [0u8; 16];
                let _ = usb_serial.read(&mut buf);
            }
        }
    };
}

#[interrupt]
fn USB() {
    poll_usb();
}
