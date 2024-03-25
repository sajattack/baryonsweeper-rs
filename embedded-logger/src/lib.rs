#![cfg_attr(not(feature="std"), no_std)]

cfg_if::cfg_if! {
    if #[cfg(feature="usb")] {

        use core::fmt::Write;
        use critical_section::Mutex;
        use core::cell::RefCell;
        use log::{Level, Record, Metadata};

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
                        let _ = write!(log, "{}: {}\r\n", record.level().as_str(), record.args());
                        let _ = usb.write(log.as_bytes());
                        let _ = usb.flush();
                        let _ = log.clear();
                    });
                }
            }

            fn flush(&self) {
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="rtt")] {
        pub use rtt_logger::RTTLogger;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature="std")] {
        static LOGGER: StdLogger = StdLogger{};

        use log::{Level, Record, LevelFilter, SetLoggerError, Metadata};

        pub struct StdLogger {}
        impl StdLogger {
            pub fn init() -> Result<(), SetLoggerError> {
                log::set_logger(&LOGGER)
                    .map(|()| log::set_max_level(LevelFilter::Trace))
            }
        }



        impl log::Log for StdLogger
        {
            fn log(&self, record: &Record) {
                println!("{} - {}", record.level(), record.args());
            }

            fn flush(&self) {
            }

            fn enabled(&self, metadata: &Metadata) -> bool {
                metadata.level() <= Level::Trace
            }


        }
    }
}
