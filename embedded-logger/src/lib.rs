//fn log(msg: &str) {
//rprintln!("{}", msg);
//let mut string = heapless::String::<256>::from_utf8(heapless::Vec::<u8, 256>::from_slice(msg.as_bytes()).unwrap()).unwrap();
//string.push_str("\r\n").unwrap();
//match self.usb_serial.write(string.as_bytes()) {
//Ok(_count) => {
//// count bytes were written
//},
//Err(UsbError::WouldBlock) => { rtt_target::rprintln!("USB buffer full") },
//Err(err) => { rtt_target::rprintln!("{:?}", err) }
//};
//}

#![no_std]

use log::{Level, Metadata, Record};

use core::borrow::BorrowMut;
use ufmt::uwrite;
use spin::RwLock;

#[cfg(feature = "rtt")]
pub use rtt_target::rprint;

cfg_if::cfg_if! {
    if #[cfg(feature="usb")] {
        pub struct UsbLogger<'a, U, const N: usize>
        where U: usb_device::bus::UsbBus
        {
            usb_serial: RwLock<usbd_serial::SerialPort<'a, U>>,
            log_buffer: RwLock<heapless::String<N>>,
        }

        impl<'a, U, const N: usize> UsbLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: usbd_serial::SerialPort<'a, U>,
            ) -> Self {
                Self {
                     usb_serial: RwLock::new(usb_serial),
                     log_buffer: RwLock::new(heapless::String::<N>::new()),
                }
            }
        }

        impl<'a, U, const N: usize> log::Log for UsbLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            fn enabled(&self, metadata: &Metadata) -> bool {
                metadata.level() <= Level::Info
            }

            fn log(&self, record: &Record) {
                if self.enabled(record.metadata()) {
                    let mut log_lock = self.log_buffer.write();
                    let mut usb_lock = self.usb_serial.write();
                    let _ = uwrite!(log_lock, "{} - {}\r\n", record.level().as_str(), record.args().as_str().unwrap());
                    let _ = usb_lock.borrow_mut().write(log_lock.as_bytes());
                }
            }

            fn flush(&self) {
                let mut usb_lock = self.usb_serial.write();
                let _ = usb_lock.flush();
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="rtt")] {
        pub struct RttLogger<const N: usize>
        {
            log_buffer: RwLock::<heapless::String<N>>,
        }

        impl<const N: usize> RttLogger<N>
        {
            pub fn new() -> Self {
                Self {
                     log_buffer: RwLock::new(heapless::String::<N>::new()),
                }
            }
        }

        impl<const N: usize> log::Log for RttLogger<N>
        {
            fn enabled(&self, metadata: &Metadata) -> bool {
                metadata.level() <= Level::Info
            }

            fn log(&self, record: &Record) {
                if self.enabled(record.metadata()) {
                    let mut log_lock = self.log_buffer.write();
                    let _ = uwrite!(log_lock, "{} - {}\r\n", record.level().as_str(), record.args().as_str().unwrap());
                    let _ = rprint!("{}", log_lock.as_str());
                }
            }

            fn flush(&self) {
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature="rtt", feature="usb"))] {
        pub struct CombinedLogger<'a, U, const N: usize>
        where U: usb_device::bus::UsbBus
        {
            usb_serial: &'a RwLock<usbd_serial::SerialPort<'a, U>>,
            log_buffer: RwLock<heapless::String<N>>,
        }

        impl<'a, U, const N: usize> CombinedLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: &'a RwLock<usbd_serial::SerialPort<'a, U>>,
            ) -> Self {
                Self {
                     usb_serial,
                     log_buffer: RwLock::new(heapless::String::<N>::new()),
                }
            }
        }

        impl<'a, U, const N: usize> log::Log for CombinedLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            fn enabled(&self, metadata: &Metadata) -> bool {
                metadata.level() <= Level::Info
            }

            fn log(&self, record: &Record) {
                if self.enabled(record.metadata()) {
                    let mut log_lock = self.log_buffer.write();
                    let mut usb_lock = self.usb_serial.write();
                    let _ = uwrite!(log_lock, "{} - {}\r\n", record.level().as_str(), record.args().as_str().unwrap());
                    let _ = rprint!("{}", log_lock.as_str());
                    let _ = usb_lock.borrow_mut().write(log_lock.as_bytes());
                }
            }

            fn flush(&self) {
                let mut usb_lock = self.usb_serial.write();
                let _ = usb_lock.flush();
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="std")] {
        pub struct StdLogger {}
        impl StdLogger {
            pub fn new() -> Self {
            }
        }

        impl log::Log for StdLogger
        {
            fn log(&self, record: &Record) {
                let _ = println!("{} - {}" record.level(), record.metadata());
            }

            fn flush(&self) {
            }
        }
    }
}
