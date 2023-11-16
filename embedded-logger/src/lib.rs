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

//use log::{Level, Metadata, Record};

use ufmt::uwrite;

#[cfg(feature = "rtt")]
pub use rtt_target::rprint;

pub trait Logger {
    fn log(&mut self, msg: &str);
    fn flush(&mut self);
}

cfg_if::cfg_if! {
    if #[cfg(feature="usb")] {
        pub struct UsbLogger<'a, U, const N: usize>
        where U: usb_device::bus::UsbBus + 'a
        {
            usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            log_buffer: heapless::String<N>,
        }

        impl<'a, U, const N: usize> UsbLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            ) -> Self {
                Self {
                     usb_serial,
                     log_buffer: heapless::String::<N>::new(),
                }
            }
        }

        impl<'a, U, const N: usize> Logger for UsbLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            fn log(&mut self, msg: &str) {
                self.log_buffer.clear();
                let _ = uwrite!(&mut self.log_buffer, "{}\r\n", msg);
                let _ = self.usb_serial.write(self.log_buffer.as_bytes());
            }

            fn flush(&mut self) {
                let _ = self.usb_serial.flush();
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="rtt")] {
        pub struct RttLogger<const N: usize>
        {
            log_buffer: heapless::String<N>,
        }

        impl<const N: usize> RttLogger<N>
        {
            pub fn new() -> Self {
                Self {
                     log_buffer: heapless::String::<N>::new(),
                }
            }
        }

        impl<const N: usize> Logger for RttLogger<N>
        {
            fn log(&mut self, msg: &str) {
                self.log_buffer.clear();
                let _ = uwrite!(&mut self.log_buffer, "{}\r\n", msg);
                let _ = rprint!("{}", self.log_buffer.as_str());
            }

            fn flush(&mut self) {
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature="rtt", feature="usb"))] {
        pub struct CombinedLogger<'a, U, const N: usize>
        where U: usb_device::bus::UsbBus + 'a
        {
            usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            log_buffer: heapless::String<N>,
        }

        impl<'a, U, const N: usize> CombinedLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            pub fn new(
                usb_serial: &'a mut usbd_serial::SerialPort<'a, U>,
            ) -> Self {
                Self {
                     usb_serial,
                     log_buffer: heapless::String::<N>::new(),
                }
            }
        }

        impl<'a, U, const N: usize> Logger for CombinedLogger<'a, U, N>
        where U: usb_device::bus::UsbBus
        {
            fn log(&mut self, msg: &str) {
                self.log_buffer.clear();
                let _ = uwrite!(&mut self.log_buffer, "{}\r\n", msg);
                let _ = self.usb_serial.write(self.log_buffer.as_bytes());
                rprint!(self.log_buffer.as_str());
            }

            fn flush(&mut self) {
                let _ = self.usb_serial.flush();
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

        impl Logger for StdLogger
        {
            fn log(&mut self, msg: &str) {
                let _ = println!("{}", msg);
            }

            fn flush(&mut self) {
            }
        }
    }
}
