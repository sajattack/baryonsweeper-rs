#![no_std]
#![no_main]

use baryonsweeper::BaryonSweeper;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;
use metro_m4 as bsp;

use bsp::hal;
use bsp::pac;
use bsp::entry;
use bsp::{pin_alias, periph_alias};

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
    let mut core = CorePeripherals::take().unwrap();
    let mut peripherals = Peripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.MCLK,
        &mut peripherals.OSC32KCTRL,
        &mut peripherals.OSCCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::Pins::new(peripherals.PORT);
    
    let gclk0 = clocks.gclk0();
    let tc2_3 = clocks.tc2_tc3(&gclk0).unwrap();
    let timer = TimerCounter::tc3_(&tc2_3, peripherals.TC3, &mut peripherals.MCLK);

    let uart_rx = pin_alias!(pins.uart_rx);
    let uart_tx = pin_alias!(pins.uart_tx);
    let uart_sercom = periph_alias!(peripherals.uart_sercom);

    let uart = bsp::uart(
        &mut clocks,
        19200.Hz(),
        uart_sercom,
        &mut peripherals.MCLK,
        uart_rx,
        uart_tx,
    );

    let bus_allocator = unsafe {
        USB_ALLOCATOR = Some(bsp::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.MCLK,
            pins.usb_dm,
            pins.usb_dp,
        ));
        USB_ALLOCATOR.as_ref().unwrap()
    };

    unsafe {
        USB_SERIAL = Some(SerialPort::new(bus_allocator));
        USB_BUS = Some(
            UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2222, 0x3333))
                .manufacturer("Sajattack")
                .product("BaryonSweeper-rs")
                .serial_number("TEST")
                .device_class(USB_CLASS_CDC)
                .build(),
        );
    }

    unsafe {
        core.NVIC.set_priority(interrupt::USB_TRCPT0, 1);
        NVIC::unmask(interrupt::USB_TRCPT0);
        core.NVIC.set_priority(interrupt::USB_TRCPT1, 1);
        NVIC::unmask(interrupt::USB_TRCPT1);
        core.NVIC.set_priority(interrupt::USB_SOF_HSOF, 1);
        NVIC::unmask(interrupt::USB_SOF_HSOF);
        core.NVIC.set_priority(interrupt::USB_OTHER, 1);
        NVIC::unmask(interrupt::USB_OTHER);
    }
    let led_pin: bsp::RedLed = pins.d13.into();

    //let usb_serial = unsafe { USB_SERIAL.as_mut().unwrap() };
    // FIXME
    //let _logger = embedded_logger::CombinedLogger::<UsbBus,256>::new(usb_serial);
    let mut baryon_sweeper = BaryonSweeper::new(uart, timer, led_pin, 500.millis());
    baryon_sweeper.sweep();
    core::unreachable!()


}

static mut USB_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut USB_BUS: Option<UsbDevice<UsbBus>> = None;
static mut USB_SERIAL: Option<SerialPort<UsbBus>> = None;

fn poll_usb() {
    unsafe {
        if let Some(usb_dev) = USB_BUS.as_mut() {
            if let Some(ref mut usb_serial) = USB_SERIAL {
                usb_dev.poll(&mut [usb_serial]);

                // Make the other side happy
                let mut buf = [0u8; 16];
                let _ = usb_serial.read(&mut buf);
            }
        }
    };
}

#[interrupt]
fn USB_TRCPT0() {
    poll_usb();
}

#[interrupt]
fn USB_TRCPT1() {
    poll_usb();
}

#[interrupt]
fn USB_SOF_HSOF() {
    poll_usb();
}

#[interrupt]
fn USB_OTHER() {
    poll_usb();
}
