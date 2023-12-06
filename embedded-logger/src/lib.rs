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

use ufmt::uwrite;
use critical_section::Mutex;
use core::cell::RefCell;

//#[cfg(feature = "rtt")]
//pub use rtt_target::rprint;

cfg_if::cfg_if! {
    if #[cfg(feature="usb")] {
        pub struct UsbLogger<'a, U, const N: usize>
        where U: usb_device::bus::UsbBus
        {
            usb_serial: Mutex<RefCell<&'a mut usbd_serial::SerialPort<'a, U>>>,
            log_buffer: Mutex<RefCell<heapless::String<N>>>,
        }

        impl<'a, U, const N: usize> UsbLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            ) -> Self {
                Self {
                     usb_serial: Mutex::new(RefCell::new(usb_serial)),
                     log_buffer: Mutex::new(RefCell::new(heapless::String::<N>::new())),
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
                    critical_section::with(|cs| {
                        let mut log = self.log_buffer.borrow_ref_mut(cs);
                        let mut usb = self.usb_serial.borrow_ref_mut(cs);
                        let _ = uwrite!(log, "{} - {}\r\n", record.level().as_str(), record.args().as_str().unwrap());
                        let _ = usb.write(log.as_bytes());
                    });
                }
            }

            fn flush(&self) {
                critical_section::with(|cs| {
                    let _ = self.usb_serial.borrow_ref_mut(cs).flush();
                });
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="rtt")] {
        pub struct RttLogger<const N: usize>
        {
            log_buffer: Mutex::<RefCell<heapless::String<N>>>,
        }

        impl<const N: usize> RttLogger<N>
        {
            pub fn new() -> Self {
                Self {
                     log_buffer: Mutex::new(RefCell::new(heapless::String::<N>::new())),
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
                    critical_section::with(|cs| {
                        //rprint!("{}", self.log_buffer.borrow_ref_mut(cs).as_str());
                    });
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
            usb_serial: Mutex<RefCell<&'a mut usbd_serial::SerialPort<'a, U>>>,
            log_buffer: Mutex<RefCell<heapless::String<N>>>,
        }

        impl<'a, U, const N: usize> CombinedLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            ) -> Self {
                Self {
                     usb_serial: Mutex::new(RefCell::new(usb_serial)),
                     log_buffer: Mutex::new(RefCell::new(heapless::String::<N>::new())),
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
                    critical_section::with(|cs| {
                        let mut log = self.log_buffer.borrow_ref_mut(cs);
                        let mut usb = self.usb_serial.borrow_ref_mut(cs);
                        let _ = uwrite!(log, "{} - {}\r\n", record.level().as_str(), record.args().as_str().unwrap());
                        let _ = usb.write(log.as_bytes());
                        //rprint!("{}", log);
                    });
                }
            }

            fn flush(&self) {
                critical_section::with(|cs| {
                    let _ = self.usb_serial.borrow_ref_mut(cs).flush();
                });
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
