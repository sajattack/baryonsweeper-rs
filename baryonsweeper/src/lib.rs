#![no_std]

use embedded_hal::{serial::{Read, Write}, timer::CountDown, digital::v2::OutputPin};
use nb::block;
use num_enum::TryFromPrimitive;
use aes::Aes128;
use aes::cipher::{
    BlockEncryptMut, KeyInit,
    generic_array::GenericArray,
};

use core::result::Result::{self, Ok, Err};
use core::option::Option::{self, Some, None};
use core::convert::{From, TryInto};
use core::unreachable;

mod consts;

use consts::*;

use log::{info, debug};

#[cfg(feature="metro_m4")]
type TimeoutType = fugit::NanosDurationU32;

#[cfg(feature="rp2040")]
type TimeoutType = fugit::MicrosDurationU64;

#[cfg(feature="itsybitsy_m0")]
type TimeoutType = itsybitsy_m0::hal::time::Nanoseconds;

pub struct BaryonSweeper<S, C, P, T> 
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    T: From<TimeoutType> + Clone,
{
    serial: S,
    timer: C,
    led_pin: P,
    timeout: T,
}
    
impl<S, C, P, T> BaryonSweeper<S, C, P, T>
where 
    S: Read<u8> + Write<u8>,
    C: CountDown,
    P: OutputPin,
    T: From<TimeoutType> + Clone,
{
    pub fn new(serial: S, timer: C, led_pin: P, timeout: T) -> BaryonSweeper<S, C, P, T> {
        Self {
            serial,
            timer,
            led_pin,
            timeout,
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
            //info!("Waiting for 5a");
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
        debug!("{}", msg.as_str());
        
    }

    fn send_packet(&mut self, code: u8, packet: &[u8]) {
        let mut msg = heapless::String::<256>::new();

        let mut full_packet = [0u8; 20];
        full_packet[0] = 0xA5;
        full_packet[1] = (packet.len() + 2) as u8;
        full_packet[2] = code;
        full_packet[3..packet.len()+3].copy_from_slice(packet);
        full_packet[packet.len() + 3] = checksum(&full_packet[0..packet.len()+3]);

        ufmt::uwrite!(msg, "Sending packet: [").unwrap();
        for i in 0..packet.len() + 3 {
            ufmt::uwrite!(msg, "0x{:02x}, ", full_packet[i]).unwrap();
            block!(self.serial.write(full_packet[i])).map_err(|_| ()).unwrap();
        }
        ufmt::uwrite!(msg, "]").unwrap();
        debug!("{}", msg.as_str());
        
    }

    pub fn sweep(&mut self) 
    where
    T: core::convert::From<TimeoutType>, <C as CountDown>::Time: From<T>
    {

        let mut recv: [u8;256];
        let mut length: u8;
        let mut challenge_version: u8 = 0;
        let mut challenge1b = [0u8; 16];

        info!("Beginning the sweep!");
        

        loop {
           //info!("Sweepin!");
           recv = [0u8;256];
           length = 0;
           self.receive_packet(&mut recv, &mut length);
           if length == 0 {
               continue;
           }

           self.led_pin.set_low().map_err(|_|()).unwrap();

           match recv[0].try_into() {
                Ok(Commands::CmdReadStatus) => {
                    info!("CmdReadStatus");
                    let response: [u8;3] = [0x10, 0xC3, 0x06];
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadTemperature) => {
                    info!("CmdReadTemperature");
                    let response: [u8; 1] = [27];
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadVoltage) => {
                    info!("CmdReadVoltage");
                    let response: [u8; 2] = [0, 0];
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadCurrent) => {
                    info!("CmdReadCurrent");
                    let current: u16 = 4200;
                    let response: [u8; 2] = current.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadCapacity) => {
                    info!("CmdReadCapacity");
                    let capacity: u16 = 1800;
                    let response: [u8; 2] = capacity.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdRead8) => {
                    info!("CmdRead8");
                    let read8: u16 = 1250; 
                    let response: [u8; 2] = read8.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadTimeLeft) => {
                    info!("CmdReadTimeLeft");
                    let time_left: u16 = 1025; 
                    let response: [u8; 2] = time_left.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response);

                },
                Ok(Commands::CmdRead11) => {
                    info!("CmdRead11");
                    let read11: u16 = 15;
                    let response: [u8; 2] = read11.to_le_bytes();
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdReadSerialno) => {
                    info!("CmdReadSerialno");
                    let sn = [SERIALNO[1], SERIALNO[0], SERIALNO[3], SERIALNO[2]];
                    self.send_packet(ResponseType::Ack as u8, &sn);
                },
                Ok(Commands::CmdRead13) => {
                    info!("CmdRead13");
                    let response: [u8; 5] = [0x9D, 0x10, 0x10, 0x28, 0x14];
                    self.send_packet(ResponseType::Ack as u8, &response);
                },
                Ok(Commands::CmdRead22) => {
                    info!("CmdRead22");
                    let response = b"SonyEnergyDevices";
                    self.send_packet(ResponseType::Ack as u8, response);
                },
                Ok(Commands::CmdAuth1) => {
                    info!("CmdAuth1");
                    challenge_version = recv[1];
                    let mut challenge1a = [0u8; 16];
                    let mut data = [0u8; 16];
                    mix_challenge1(challenge_version, &recv[2..], &mut data).unwrap();
                    encrypt_bytes(&data, challenge_version, &mut challenge1a).unwrap();
                    let second = challenge1a.clone();
                    let mut temp = [0u8; 16];
                    encrypt_bytes(&second, challenge_version, &mut temp).unwrap();
                    matrix_swap(&temp, &mut challenge1b);
                    let mut packet = [0u8; 16];
                    packet[0..8].copy_from_slice(&challenge1a[0..8]);
                    packet[8..16].copy_from_slice(&challenge1b[0..8]);
                    self.send_packet(ResponseType::Ack as u8, &packet);
                },
                Ok(Commands::CmdAuth2) => {
                    info!("CmdAuth2");
                    let mut data2 = [0u8; 16];
                    let mut challenge2 = [0u8; 16];
                    let mut temp = [0u8; 16];
                    let mut packet = [0u8; 16];
                    mix_challenge2(challenge_version, &challenge1b[0..8], &mut temp).unwrap();
                    matrix_swap(&temp, &mut data2);
                    encrypt_bytes(&data2, challenge_version, &mut challenge2).unwrap();
                    encrypt_bytes(&challenge2, challenge_version, &mut packet).unwrap();
                    self.send_packet(ResponseType::Ack as u8, &packet);

                },
                _ => {
                    self.send_packet(ResponseType::Nak as u8, &[]);   
                        info!("Sending General NAK!");
                }           
           }

           self.led_pin.set_high().map_err(|_|()).unwrap();

        }
    }

}


fn mix_challenge1(version: u8, challenge: &[u8], data: &mut [u8]) -> Result<(), ()>
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

fn mix_challenge2(version: u8, challenge: &[u8], data: &mut [u8]) -> Result<(), ()>
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
        let secret2 = secret2.unwrap();
        data[0x00] = challenge[0x00];
        data[0x04] = challenge[0x01];
        data[0x08] = challenge[0x02];
        data[0x0C] = challenge[0x03];
        data[0x01] = challenge[0x04];
        data[0x05] = challenge[0x05];
        data[0x09] = challenge[0x06];
        data[0x0D] = challenge[0x07];
        data[0x02] = secret2[0x00];
        data[0x06] = secret2[0x01];
        data[0x0A] = secret2[0x02];
        data[0x0E] = secret2[0x03];
        data[0x03] = secret2[0x04];
        data[0x07] = secret2[0x05];
        data[0x0B] = secret2[0x06];
        data[0x0F] = secret2[0x07];
        Ok(())
    }
}

fn encrypt_bytes(plain_bytes: &[u8; 16], version: u8, encrypted: &mut [u8]) -> Result<(), ()>
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
        let mut ctx = ecb::Encryptor::<Aes128>::new(&GenericArray::from(key.unwrap()));
        let block = GenericArray::from(*plain_bytes);
        let mut encrypted_gen = GenericArray::from_mut_slice(encrypted);
        ctx.encrypt_block_b2b_mut(&block, &mut encrypted_gen);
        Ok(())
    }
}

const NEW_MAP: [usize; 16] = [
    0x00, 0x04, 0x08, 0x0C, 0x01, 0x05, 0x09, 0x0D, 0x02, 0x06, 0x0A, 0x0E, 0x03, 0x07, 0x0B, 0x0F,
];

fn matrix_swap(key: &[u8], out: &mut [u8]) {
    for i in 0..key.len() {
        out[i] = key[NEW_MAP[i]];
    }
}

fn checksum(packet: &[u8]) -> u8 {
    let sh: u16 = packet.iter().map(|n| *n as u16).sum();
    return (0xFFu16 - (sh & 0xffu16)) as u8;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_response_cmdauth1() {
        let challenge: [u8; 13] = [0x5A, 0x0B, 0x80, 0xD9, 0x8E, 0x35, 0xF3, 0x8F, 0x2B, 0x8C, 0x6D, 0x8F, 0x49];
        let expected_response: [u8; 20] = [
            0xA5, 0x12, 0x06, 0x83, 0x32, 0x32, 0xDE, 0xF3, 0x25, 0xA2,
            0x7C, 0x1A, 0xC9, 0x21, 0x7A, 0xE9, 0x8F, 0xBE, 0x22, 0x71
        ];

        let code: u8 = ResponseType::Ack as u8;

        let challenge_version = challenge[3];
        let mut challenge1a = [0u8; 16];
        let mut challenge1b = [0u8; 16];
        let mut data = [0u8; 16];
        let mut temp = [0u8; 16];
        let _ = mix_challenge1(challenge_version, &challenge[4..], &mut data);
        let _ = encrypt_bytes(&data, challenge_version, &mut challenge1a);
        let second = challenge1a.clone();
        let _ = encrypt_bytes(&second, challenge_version, &mut temp);
        matrix_swap(&temp, &mut challenge1b);
        let mut packet = [0u8; 16];
        packet[0..8].copy_from_slice(&challenge1a[0..8]);
        packet[8..16].copy_from_slice(&challenge1b[0..8]);
        let temp = [
            0xA5, 16 + 2, code, packet[0], packet[1], packet[2], packet[3], packet[4], packet[5], packet[6], packet[7],
            packet[8], packet[9], packet[10], packet[11], packet[12], packet[13], packet[14], packet[15]];
        assert_eq!(checksum(&temp), expected_response[19]);
        let mut send = [0u8; 20];
        send[0..19].copy_from_slice(&temp);
        send[19] = checksum(&temp);
        assert_eq!(expected_response, send);
    }

    #[test]
    fn test_challenge_response_cmdauth2() {
        let _challenge: [u8; 12] = [0x5A, 0x0A, 0x81, 0x13, 0xF1, 0x06, 0x0B, 0x97, 0x9E, 0x9F, 0xF9, 0x38];
        let expected_response: [u8;20] = [
            0xA5, 0x12, 0x06, 0xBA, 0x54, 0x76, 0x57, 0x8E, 0xAF, 0x4E,
            0x8F, 0xAD,  0xF2, 0xA3, 0x55, 0xDA, 0x10, 0xC2, 0x1D, 0xED
        ];


        let code: u8 = ResponseType::Ack as u8;
        let challenge_version = 0xD9;

        let mut data2 = [0u8; 16];
        let mut temp = [0u8; 16];
        let mut challenge2 = [0u8; 16];
        let mut packet = [0u8; 16];
        let challenge1b: [u8; 8] = [0x1A, 0xC9, 0x21, 0x7A, 0xE9, 0x8F, 0xBE, 0x22];
        mix_challenge2(challenge_version, &challenge1b[0..8], &mut temp).unwrap();
        matrix_swap(&temp, &mut data2);
        encrypt_bytes(&data2, challenge_version, &mut challenge2).unwrap();
        encrypt_bytes(&challenge2, challenge_version, &mut packet).unwrap();



        let temp = [
            0xA5, 16 + 2, code, 
            packet[0], packet[1], packet[2], packet[3], packet[4], packet[5], packet[6], packet[7],
            packet[8], packet[9], packet[10], packet[11], packet[12], packet[13], packet[14], packet[15],

        ]; 
        assert_eq!(checksum(&temp), expected_response[19]);
        let mut send = [0u8; 20];
        send[0..19].copy_from_slice(&temp);
        send[19] = checksum(&temp);
        assert_eq!(expected_response, send);

    }
}
