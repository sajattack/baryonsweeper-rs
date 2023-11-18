#![no_std]

use embedded_hal::{serial::{Read, Write}, timer::CountDown, digital::v2::OutputPin};
use nb::block;
use num_enum::TryFromPrimitive;
use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, KeyInit,
    generic_array::GenericArray,
};
use embedded_logger::Logger;

use core::result::Result::{self, Ok, Err};
use core::option::Option::{self, Some, None};
use core::convert::{From, TryInto};
use core::unreachable;

mod consts;

use consts::*;

//use log::{info, debug};

#[cfg(feature="metro_m4")]
type TimeoutType = fugit::NanosDurationU32;

#[cfg(feature="rp2040")]
type TimeoutType = fugit::MicrosDurationU64;

#[cfg(feature="itsybitsy_m0")]
type TimeoutType = itsybitsy_m0::hal::time::Nanoseconds;

pub struct BaryonSweeper<S, C, P, T, L> 
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    T: From<TimeoutType> + Clone,
    L: Logger,
{
    serial: S,
    timer: C,
    led_pin: P,
    timeout: T,
    logger: L,
}
    
impl<S, C, P, T, L> BaryonSweeper<S, C, P, T, L>
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    T: From<TimeoutType> + Clone,
    L: Logger,
{
    pub fn new(serial: S, timer: C, led_pin: P, timeout: T, logger: L) -> BaryonSweeper<S, C, P, T, L> {
        Self {
            serial,
            timer,
            led_pin,
            timeout,
            logger,
        }
    }

    fn mix_challenge1(&self, version: u8, challenge: &[u8], data: &mut [u8]) -> Result<(), ()>
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
                data[8+i] = challenge[i];
            }
            Ok(())
        }
    }

    fn mix_challenge2(&self, version: u8, challenge: &[u8], data: &mut [u8]) -> Result<(), ()>
    {
        let mut secret2: Option<[u8;8]> = None;
        for i in 0..SECRETS2.len() {
            if SECRETS2[i].version == version {
                secret2 = Some(SECRETS2[i].secret);
            }
        }
        if secret2.is_none() {
            Err(())
        } else {
            for i in 0..8 {
                data[i] = secret2.unwrap()[i];
            }
            for i in 0..8 {
                data[8+i] = challenge[i];
            }
            Ok(())
        }
    }

    fn encrypt_bytes(&self, plain_bytes: &[u8; 16], version: u8, enc_bytes: &mut [u8]) -> Result<(), ()>
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
            let mut block = GenericArray::from(*plain_bytes);
            ctx.encrypt_block(&mut block);
            enc_bytes.copy_from_slice(&block); 
            Ok(())
        }
    }

    fn read_with_timeout
        (
            &mut self,
            timeout: T,
        ) -> Result<u8, ()>
        where
        T: core::convert::From<TimeoutType> ,<C as CountDown>::Time: From<T>
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
    T: core::convert::From<TimeoutType> ,<C as CountDown>::Time: From<T>
    {
        loop {
            //self.logger.log("Waiting for 5a");
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
        //self.logger.log(msg.as_str());
        //self.logger.flush();
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
        //self.logger.log(msg.as_str());
        //self.logger.flush();
    }


    pub fn sweep(&mut self) 
    where
    T: core::convert::From<TimeoutType>, <C as CountDown>::Time: From<T>
    {

        let mut recv: [u8;256];
        let mut length: u8;
        let mut challenge_version: u8 = 0;
        let mut challenge1b = [0u8; 16];

        self.logger.log("Beginning the sweep!");
        self.logger.flush();

        loop {
           //self.logger.log("Sweepin!");
           recv = [0u8;256];
           length = 0;
           self.receive_packet(&mut recv, &mut length);
           if length == 0 {
               continue;
           }

           self.led_pin.set_low().map_err(|_|()).unwrap();

           match recv[0].try_into() {
                Ok(Commands::CmdReadStatus) => {
                    self.logger.log("CmdReadStatus");
                    self.logger.flush();
                    let response: [u8;3] = [0x10, 0xC3, 0x06];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadTemperature) => {
                    self.logger.log("CmdReadTemperature");
                    self.logger.flush();
                    let response: [u8; 1] = [27];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadVoltage) => {
                    self.logger.log("CmdReadVoltage");
                    self.logger.flush();
                    let response: [u8; 2] = [0, 0];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadCurrent) => {
                    self.logger.log("CmdReadCurrent");
                    self.logger.flush();
                    let current: u16 = 4200;
                    let response: [u8; 2] = current.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadCapacity) => {
                    self.logger.log("CmdReadCapacity");
                    self.logger.flush();
                    let capacity: u16 = 1800;
                    let response: [u8; 2] = capacity.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdRead8) => {
                    self.logger.log("CmdRead8");
                    self.logger.flush();
                    let read8: u16 = 1250; 
                    let response: [u8; 2] = read8.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadTimeLeft) => {
                    self.logger.log("CmdReadTimeLeft");
                    self.logger.flush();
                    let time_left: u16 = 1025; 
                    let response: [u8; 2] = time_left.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());

                },
                Ok(Commands::CmdRead11) => {
                    self.logger.log("CmdRead11");
                    self.logger.flush();
                    let read11: u16 = 15;
                    let response: [u8; 2] = read11.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdReadSerialno) => {
                    self.logger.log("CmdReadSerialno");
                    self.logger.flush();
                    let sn = [SERIALNO[1], SERIALNO[0], SERIALNO[3], SERIALNO[2]];
                    self.send_packet(ResponseType::Ack as u8, &sn, sn.len());
                },
                Ok(Commands::CmdRead13) => {
                    self.logger.log("CmdRead13");
                    self.logger.flush();
                    let response: [u8; 5] = [0x9D, 0x10, 0x10, 0x28, 0x14];
                    self.send_packet(ResponseType::Ack as u8, &response, response.len());
                },
                Ok(Commands::CmdRead22) => {
                    self.logger.log("CmdRead22");
                    self.logger.flush();
                    let response = b"SonyEnergyDevices";
                    self.send_packet(ResponseType::Ack as u8, response, response.len());
                },
                Ok(Commands::CmdAuth1) => {
                    self.logger.log("CmdAuth1");
                    self.logger.flush();
                    challenge_version = recv[1];
                    let mut challenge1a = [0u8; 16];
                    let mut data = [0u8; 16];
                    let _ = self.mix_challenge1(challenge_version, &recv[2..], &mut data);
                    let _ = self.encrypt_bytes(&data, challenge_version, &mut challenge1a);
                    let second = challenge1a.clone();
                    let _ = self.encrypt_bytes(&second, challenge_version, &mut challenge1b);
                    let mut packet = [0u8; 16];
                    packet[0..8].copy_from_slice(&challenge1a[0..8]);
                    packet[8..16].copy_from_slice(&challenge1b[0..8]);
                    self.send_packet(ResponseType::Ack as u8, &packet, packet.len());
                },
                Ok(Commands::CmdAuth2) => {
                    self.logger.log("CmdAuth2");
                    self.logger.flush();
                    let mut data2 = [0u8; 16];
                    let mut challenge2 = [0u8; 16];
                    let mut packet = [0u8; 16];
                    let _ = self.mix_challenge2(challenge_version, &challenge1b[0..8], &mut data2);
                    let _ = self.encrypt_bytes(&data2, challenge_version, &mut challenge2);
                    let _ = self.encrypt_bytes(&challenge2, challenge_version, &mut packet);
                    self.send_packet(ResponseType::Ack as u8, &packet[0..8], 8);
                    if challenge_version == 0xeb || challenge_version == 0xb3 {
                        let special_packet = [0x5a, 0x02, 0x01, 0xa2];
                        for byte in special_packet {
                            block!(self.serial.write(byte));
                        }
                    }
                },
                _ => {
                    let mut msg = heapless::String::<256>::new();
                    ufmt::uwrite!(msg, "Unknown packet: 0x{:02x}", recv[0]).unwrap();
                    self.logger.log(&msg); 
                    self.logger.flush();
                    self.send_packet(ResponseType::Nak as u8, &[0], 0);   
                    self.logger.log("Sending General NAK!");
                    self.logger.flush();

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
