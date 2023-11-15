#![no_std]

use embedded_hal::{serial::{Read, Write}, timer::CountDown, digital::v2::OutputPin};
//use fugit::ExtU64;
use nb::block;
use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, KeyInit,
    generic_array::GenericArray,
};
use usb_device::{class_prelude::UsbBus, UsbError};
use usbd_serial::SerialPort;
use num_enum::TryFromPrimitive;

use rtt_target::{rprint, rprintln};

use core::result::Result::{self, Ok, Err};
use core::option::Option::{self, Some, None};
use core::convert::{From, TryInto};
use core::unreachable;

mod consts;

use consts::*;

pub struct BaryonSweeper<'a, S, C, P, U, T> 
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    U: UsbBus,
    T: From<fugit::MicrosDurationU64> + Clone,
{
    serial: S,
    timer: C,
    led_pin: P,
    usb_serial: &'a mut SerialPort<'a, U>,
    timeout: T,
}
    
impl<'a, S, C, P, U, T> BaryonSweeper<'a, S, C, P, U, T>
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    U: UsbBus,
    T: From<fugit::MicrosDurationU64> + Clone,
{
    pub fn new(serial: S, timer: C, led_pin: P, usb_serial: &'a mut SerialPort<'a, U>, timeout: T) -> BaryonSweeper<'a, S, C, P, U, T> {
        Self {
            serial,
            timer,
            led_pin,
            usb_serial,
            timeout,
        }
    }

    fn mix_challenge1(&self, version: u8, challenge: &[u8; 16], data: &mut [u8; 16]) -> Result<(), ()>
    {
        let mut secret1: Option<[u8;8]> = None;
        for i in 0..SECRETS1.len() {
            if SECRETS1[i].version == version {
                secret1 = Some(SECRETS1[i].secret);
            }
        }
        if secret1.is_none() {
            Err(())
        } else {
            for i in 0..8 {
                data[i] = secret1.unwrap()[i];
            }
            for i in 0..8 {
                data[7+i] = challenge[i];
            }
            Ok(())
        }
    }

    fn mix_challenge2(&self, version: u8, challenge: &[u8; 16], data: &mut [u8; 16]) -> Result<(), ()>
    {
        let mut secret2: Option<[u8;8]> = None;
        for i in 0..SECRETS2.len() {
            if SECRETS2[i].version == version {
                secret2 = Some(SECRETS1[i].secret);
            }
        }
        if secret2.is_none() {
            Err(())
        } else {
            for i in 0..8 {
                data[i] = secret2.unwrap()[i];
            }
            for i in 0..8 {
                data[7+i] = challenge[i];
            }
            Ok(())
        }
    }

    fn encrypt_bytes(&self, plain_bytes: &[u8; 16], version: u8, _enc_bytes: &mut [u8; 16]) -> Result<(), ()>
    {
        let mut key: Option<[u8;16]> = None;
        for i in 0..KEYS.len() {
            if KEYS[i].version == version {
                key = Some(KEYS[i].key);
            }
        }
        if key.is_none() {
            Err(())
        } else {
            let ctx = Aes128::new(&GenericArray::from(key.unwrap()));
            ctx.encrypt_block(&mut GenericArray::from(*plain_bytes));
            Ok(())
        }
    }

    fn generate_response(&self, req: &[u8;16], resp: &mut [u8; 16], version: u8) -> Result<(), ()>
    {
        let mut data: [u8; 16] = [0u8;16];
        if self.mix_challenge1(version, req, &mut data).is_err() {
            return Err(());
        }
        if self.encrypt_bytes(&data.clone(), version, &mut data).is_err() {
            return Err(());
        }
        resp[0..8].copy_from_slice(&data[0..8]);
        Ok(())
    }

    fn check_response(&self, req: &[u8;16], resp: &mut [u8;16], version: u8) -> Result<(), ()>
    {
        let mut data: [u8; 16] = [0u8;16];
        if self.mix_challenge2(version, req, &mut data).is_err() {
            return Err(());
        }
        if self.encrypt_bytes(&data.clone(), version, &mut data).is_err() {
            return Err(());
        }
        if req[0..8] != data[0..8]
        {
            return Err(());
        }
        // Why do we need to encrypt twice and why is it an error for req != data at this point?
        if self.encrypt_bytes(&data.clone(), version, &mut data).is_err() {
            return Err(());
        }
        resp[0..8].copy_from_slice(&data[0..8]);
        Ok(())
    }


    fn read_with_timeout
        (
            &mut self,
            timeout: T,
        ) -> Result<u8, ()>
        where
        T: core::convert::From<fugit::MicrosDurationU64> ,<C as CountDown>::Time: From<T>
    {
        self.timer.start(timeout);

        loop {
            match self.serial.read() {
                // raise error
                Err(nb::Error::Other(_e)) => return Err(()),//return Err(Error::Serial(e)),
                Err(nb::Error::WouldBlock) => {
                    // no data available yet, check the timer below
                },
                Ok(byte) => return Ok(byte),
            }

            match self.timer.wait() {
                Err(nb::Error::Other(_e)) => {
                    // The error type specified by `timer.wait()` is `!`, which
                    // means no error can actually occur. The Rust compiler
                    // still forces us to provide this match arm, though.
                    unreachable!()
                },
                // no timeout yet, try again
                Err(nb::Error::WouldBlock) => continue,
                Ok(()) => return Err(()),//return Err(Error::TimedOut),
            }
        }
    }


    fn receive_packet(&mut self, recv: &mut [u8], len: &mut u8)
    where
    T: core::convert::From<fugit::MicrosDurationU64> ,<C as CountDown>::Time: From<T>
    {
        loop {
            self.log("Waiting for 5a");
            if let Ok(0x5a) = block!(self.serial.read())  {
                break;
            }
        }
        let length = block!(self.serial.read()).map_err(|_|()).unwrap();
        *len = length-1;

        let mut msg = heapless::String::<256>::new();
        ufmt::uwrite!(msg, "Received packet: [0x5a, 0x{:02x}, ", length).unwrap();
        for i in 0..length {
            let res = self.read_with_timeout(self.timeout.clone());
            if res.is_err() {
                *len = 0;
                return;
            }
            else {
                recv[i as usize] = res.unwrap();
                ufmt::uwrite!(msg, "0x{:02x}, ", recv[i as usize]).unwrap();
            }
        }
        ufmt::uwrite!(msg, "]").unwrap();
        self.log(msg.as_str());
    }

    fn send_packet(&mut self, code: u8, packet: &[u8], length: usize) {
        let mut msg = heapless::String::<256>::new();
        ufmt::uwrite!(msg, "Sending packet: [0xA5, 0x{:02x}, 0x{:02x}, ", length+2, code).unwrap();

        block!(self.serial.write(0xA5)).map_err(|_| ()).unwrap();
        block!(self.serial.write(length as u8 + 2)).map_err(|_| ()).unwrap();
        block!(self.serial.write(code)).map_err(|_| ()).unwrap();
        let mut sum: u8 = 0xA5 + code + length as u8 + 2;
        for i in 0..length {
            ufmt::uwrite!(msg, "0x{:02x}, ", packet[i]).unwrap();
            block!(self.serial.write(packet[i])).map_err(|_| ()).unwrap();
            sum += packet[i];
        }
        block!(self.serial.write(!sum)).map_err(|_| ()).unwrap();
        ufmt::uwrite!(msg, "0x{:02x}]", !sum).unwrap();
        self.log(msg.as_str());
    }

    fn log(&mut self, msg: &str) {
        rprintln!("{}", msg);
        let mut string = heapless::String::<256>::from_utf8(heapless::Vec::<u8, 256>::from_slice(msg.as_bytes()).unwrap()).unwrap();
        string.push_str("\r\n").unwrap();
        match self.usb_serial.write(string.as_bytes()) {
            Ok(_count) => {
                // count bytes were written
            },
            Err(UsbError::WouldBlock) => { rtt_target::rprintln!("USB buffer full") },
            Err(err) => { rtt_target::rprintln!("{:?}", err) }
        };
    }

    pub fn sweep(&mut self) 
    where
    T: core::convert::From<fugit::MicrosDurationU64>, <C as CountDown>::Time: From<T>
    {

        let mut recv: [u8;256];
        let mut length: u8;
        let mut challenge_version: u8;

        self.log("Beginning the sweep!");

        loop {
           self.log("Sweepin!");
           recv = [0u8;256];
           length = 0;
           self.receive_packet(&mut recv, &mut length);
           if length == 0 {
               continue;
           }

           self.led_pin.set_low().map_err(|_|()).unwrap();

           match recv[0].try_into() {
                Ok(Commands::CmdReadStatus) => {
                    let response: [u8;3] = [0x10, 0xC3, 0x06];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadTemperature) => {
                    let response: [u8; 1] = [27];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadVoltage) => {
                    let response: [u8; 2] = [0, 0];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadCurrent) => {
                    let current: u16 = 4200;
                    let response: [u8; 2] = current.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadCapacity) => {
                    let capacity: u16 = 1800;
                    let response: [u8; 2] = capacity.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdRead8) => {
                    let read8: u16 = 1250; 
                    let response: [u8; 2] = read8.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadTimeLeft) => {
                    let time_left: u16 = 1025; 
                    let response: [u8; 2] = time_left.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());

                },
                Ok(Commands::CmdRead11) => {
                    let read11: u16 = 15;
                    let response: [u8; 2] = read11.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadSerialno) => {
                    let sn = [SERIALNO[1], SERIALNO[0], SERIALNO[3], SERIALNO[2]];
                    self.send_packet(ResponseType::Ack as u8, &sn, sn.len());
                },
                Ok(Commands::CmdRead13) => {
                    let response: [u8; 5] = [0x9D, 0x10, 0x10, 0x28, 0x14];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdRead22) => {
                    let response = b"SonyEnergyDevices";
                    self.send_packet(ResponseType::Ack as u8, response, response.len());
                },
                Ok(Commands::CmdAuth1) => {
                    challenge_version = recv[1];
                    let mut challenge_response = [0u8; 16];
                    let mut challenge_request = [0u8; 16];
                    challenge_request[0] = recv[2];
                    if self.generate_response(&challenge_request, &mut challenge_response, challenge_version).is_ok()
                    {
                        let mut packet = [0u8;16];
                        packet[0..8].copy_from_slice(&challenge_response[0..8]);
                        packet[8..16].copy_from_slice(&BATTERY_NONCE);
                        self.send_packet(ResponseType::Ack as u8, &packet, packet.len());
                    } else {
                        self.send_packet(ResponseType::Nak as u8, &[0], 0);   
                    }
                },
                Ok(Commands::CmdAuth2) => {
                    challenge_version = recv[1];
                    let mut challenge_response = [0u8; 16];
                    let mut challenge_request = [0u8; 16];
                    challenge_request[0] = recv[1];
                    if self.check_response(&challenge_request, &mut challenge_response, challenge_version).is_ok()
                    {
                        self.send_packet(ResponseType::Ack as u8, &challenge_response, challenge_response.len());
                    } else {
                        self.send_packet(ResponseType::Nak as u8, &[0], 0);   
                    }
                },
                _ => {
                    self.send_packet(ResponseType::Nak as u8, &[0], 0);   
                }

           }

           self.led_pin.set_high().map_err(|_|()).unwrap();

        }
    }

}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
enum Commands {
    CmdReadStatus = 1,
    CmdReadTemperature,
    CmdReadVoltage,
    CmdReadCurrent,
    CmdReadCapacity = 7,
    CmdRead8,
    CmdReadTimeLeft,
    CmdRead11 = 11,
    CmdReadSerialno,
    CmdRead13,
    CmdWriteEeprom = 19,
    CmdReadEeprom,
    CmdRead22 = 22,
    CmdAuth1 = 0x80,
    CmdAuth2,
}

#[repr(u8)]
enum ResponseType {
    Nak = 5,
    Ack,
}
