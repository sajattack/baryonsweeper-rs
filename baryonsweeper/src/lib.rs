#![cfg_attr(not(feature="std"), no_std)]

use embedded_hal::{serial::{Read, Write}, timer::CountDown, digital::v2::OutputPin};
use nb::block;
use num_enum::TryFromPrimitive;
use aes::Aes128;
use aes::cipher::{
    KeyIvInit, BlockDecryptMut, BlockEncryptMut, KeyInit, 
    generic_array::GenericArray,
};
use ufmt::uWrite;
use core::convert::{From, TryInto};
use core::unreachable;

mod consts;

use consts::*;

#[cfg(any(feature="std", feature="usb"))]
use log::{info, debug};
#[cfg(feature="rtt")]
use defmt::{info, debug};

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


    fn receive_packet(&mut self, recv: &mut [u8; 64], len: &mut u8)
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

        for i in 0..length {
            let res = self.read_with_timeout(self.timeout.clone());
            if res.is_err() {
                *len = 0;
                return;
            }
            else {
                recv[i as usize] = res.unwrap();
            }
        }
       
        
        #[cfg(debug_assertions)]
        {
            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Received packet: 0x5a, 0x{:02X} ", length).unwrap();
            let _ = msg.write_str(fmt_packet(recv, length.into()).as_str());
            debug!("{}", msg.as_str());
        }
    }


    fn send_packet(&mut self, packet: &[u8], size: usize) {
        #[cfg(debug_assertions)] 
        {
            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet(packet, size).as_str());
            debug!("{}", msg.as_str());
        }
        
        for i in 0..size {
            let _ = block!(self.serial.write(packet[i])).map_err(|_|());
        }
    }

    pub fn sweep(&mut self) 
    where
    T: core::convert::From<TimeoutType>, <C as CountDown>::Time: From<T>
    {

        let mut length: u8;
        let mut challenge_version: u8 = 0;
        let mut challenge1b = [0u8; 16];

        info!("Beginning the sweep!");
        

        loop {
           let mut recv = [0u8;64];
           length = 0;
           self.receive_packet(&mut recv, &mut length);
           if length == 0 {
               continue;
           }

           self.led_pin.set_low().map_err(|_|()).unwrap();

           match recv[0].try_into() {
                Ok(Commands::CmdReadStatus) => {
                    let response = cmd_read_status();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadTemperature) => {
                    let response = cmd_read_temperature();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadVoltage) => {
                    let response = cmd_read_voltage();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadCurrent) => {
                    let response = cmd_read_current();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadCapacity) => {
                    let response = cmd_read_capacity();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdRead8) => {
                    let response = cmd_read8();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadTimeLeft) => {
                    let response = cmd_read_time_left();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);

                },
                Ok(Commands::CmdRead11) => {
                    let response = cmd_read11();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdReadSerialno) => {
                    let response = cmd_read_serialno();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdRead13) => {
                    let response = cmd_read13();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdRead22) => {
                    let response = cmd_read22();
                    let packet = build_packet(ResponseType::Ack as u8, &response);
                    self.send_packet(&packet.0, packet.1);
                },
                Ok(Commands::CmdAuth1) => {
                    challenge_version = recv[1];
                    let challenge = &recv[2..];
                    if let Ok((response, bchal)) = cmdauth1(challenge_version, challenge)
                    {
                        challenge1b = bchal;
                        let packet = build_packet(ResponseType::Ack as u8, &response);
                        self.send_packet(&packet.0, packet.1);
                    }
                    else {
                        let response = [0xff; 8];
                        let packet = build_packet(ResponseType::Ack as u8, &response);
                        self.send_packet(&packet.0, packet.1); 
                    }
                    info!("Challenge version: 0x{:x}", challenge_version);
                },
                Ok(Commands::CmdAuth2) => {
                    let challenge = &recv[2..];
                    if let Ok(response) = cmdauth2(challenge_version, challenge, &challenge1b)
                    {
                        let packet = build_packet(ResponseType::Ack as u8, &response);
                        self.send_packet(&packet.0, packet.1);
                    }
                    info!("Challenge version: 0x{:x}", challenge_version);
                    if challenge_version == 0xeb || challenge_version == 0xb3 {
                        let mut packet2 = [0u8; 64];
                        packet2[0..4].copy_from_slice(&[0x5a, 0x02, 0x01, 0xa2]);
                        self.send_packet(&packet2, 4);
                    }
                },
                Ok(Commands::CmdAuthGo) => {
                    let screq = &recv[3..];
                    if let Ok(response) = cmdauthgo(screq)
                    {
                        let packet = build_packet(ResponseType::Ack as u8, &response);
                        self.send_packet(&packet.0, packet.1);
                    }
                    else 
                    {
                        info!("CmdAuthGo returned error")
                    }
                },
                _ => {
                    let packet = build_packet(ResponseType::Nak as u8, &[]);
                    self.send_packet(&packet.0, packet.1);
                        info!("Sending General NAK!");
                }           
           }

           self.led_pin.set_high().map_err(|_|()).unwrap();

        }
    }

}

fn cmd_read_status() -> [u8;3] {
    info!("CmdReadStatus");
    [0x10, 0xc3, 0x06]
}

fn cmd_read_temperature() -> [u8;1] {
    info!("CmdReadTemperature");
    [27]
}

fn cmd_read_voltage() -> [u8;2] {
    info!("CmdReadVoltage");
    [0x36, 0x10]
}

fn cmd_read_current() -> [u8;2] {
    info!("CmdReadCurrent");
    let current: u16 = 4200;
    current.to_le_bytes()
}

fn cmd_read_capacity() -> [u8;2] {
    info!("CmdReadCapacity");
    let capacity: u16 = 1800;
    capacity.to_le_bytes()
}

fn cmd_read8() -> [u8;2] {
    info!("CmdRead8");
    let read8: u16 = 1250; 
    read8.to_le_bytes()
}

fn cmd_read_time_left() -> [u8;2] {
    info!("CmdReadTimeLeft");
    let time_left: u16 = 1025;
    time_left.to_le_bytes()
}

fn cmd_read11() -> [u8;2] {
    info!("CmdRead11");
    let read11: u16 = 15;
    read11.to_le_bytes()
}

fn cmd_read_serialno() -> [u8; 4] {
    info!("CmdReadSerialno");
    [SERIALNO[1], SERIALNO[0], SERIALNO[3], SERIALNO[2]]
}

fn cmd_read13() -> [u8; 5] {
    info!("CmdRead13");
    [0x9d, 0x10, 0x10, 0x28, 0x14]
}

fn cmd_read22() -> [u8; 17]
{
    info!("CmdRead22");
    *b"SonyEnergyDevices"
}

fn cmdauth1(version: u8, challenge: &[u8]) -> Result<([u8; 16], [u8; 16]), ()> {
    info!("CmdAuth1");
    let mut challenge1a = [0u8; 16];
    let mut challenge1b = [0u8; 16];
    let mut data = [0u8; 16];

    #[cfg(debug_assertions)]
    let mut msg = heapless::String::<1024>::new();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "challenge: ");
        msg.write_str(fmt_packet(challenge, challenge.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    if mix_challenge1(version, challenge, &mut data).is_err() {
       return Err(())
    }

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "mixed: ");
        msg.write_str(fmt_packet(&data, data.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    encrypt_bytes(&data, version, &mut challenge1a).unwrap();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "ch1a: ");
        msg.write_str(fmt_packet(&challenge1a, challenge1a.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    let second = challenge1a;
    let mut temp = [0u8; 16];
    encrypt_bytes(&second, version, &mut temp).unwrap();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "temp: ");
        msg.write_str(fmt_packet(&temp, temp.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    matrix_swap(&temp, &mut challenge1b);

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "ch1b: ");
        msg.write_str(fmt_packet(&challenge1b, challenge1b.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    let mut packet = [0u8; 16];
    packet[0..8].copy_from_slice(&challenge1a[0..8]);
    packet[8..16].copy_from_slice(&challenge1b[0..8]);

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "AB: ");
        msg.write_str(fmt_packet(&packet, packet.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    Ok((packet, challenge1b))
}

fn cmdauth2(challenge_version: u8, _challenge: &[u8], ch1b: &[u8]) -> Result<[u8; 16], ()>
{
    info!("CmdAuth2");
    let mut data2 = [0u8; 16];
    let mut challenge2 = [0u8; 16];
    let mut temp = [0u8; 16];
    let mut packet = [0u8; 16];

    #[cfg(debug_assertions)]
    let mut msg = heapless::String::<1024>::new();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "ch1b: ");
        msg.write_str(fmt_packet(&ch1b, ch1b.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    mix_challenge2(challenge_version, &ch1b[0..8], &mut temp).unwrap();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "mixed: ");
        msg.write_str(fmt_packet(&temp, temp.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    matrix_swap(&temp, &mut data2);

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "swapped: ");
        msg.write_str(fmt_packet(&data2, data2.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    encrypt_bytes(&data2, challenge_version, &mut challenge2).unwrap();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "enc1: ");
        msg.write_str(fmt_packet(&challenge2, challenge2.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    encrypt_bytes(&challenge2, challenge_version, &mut packet).unwrap();

    #[cfg(debug_assertions)]
    {
        let _ = ufmt::uwrite!(msg, "enc2: ");
        msg.write_str(fmt_packet(&packet, packet.len()).as_str());
        debug!("{}", msg.as_str());
        msg.clear();
    }

    Ok(packet)
}

fn cmdauthgo(screq: &[u8]) -> Result<[u8; 40], ()>
{
    info!("CmdAuthGo");
    let mut enc = [[0u8; 16]; 2];
    enc[0].copy_from_slice(&screq[8..24]);
    enc[1].copy_from_slice(&screq[24..40]);
    let key = GenericArray::from(GO_KEY1);
    let iv = GenericArray::from([0u8; 16]);

    let mut decryptor = cbc::Decryptor::<Aes128>::new(&key, &iv);
    let block1 = GenericArray::from(enc[0]);
    let block2 = GenericArray::from(enc[1]);
    let mut blocks = [block1, block2];
    decryptor.decrypt_blocks_mut(&mut blocks);


    let decrypted = blocks.as_slice();

    if decrypted[1].as_slice() == GO_SECRET {
        info!("Go handshake request is valid");
    } else {
        info!("Invalid request from Syscon");
        return Err(())
    }

    let mut response_payload = [[0u8; 16]; 2];
    response_payload[0][0..8].copy_from_slice(&decrypted[0][8..16]);
    response_payload[0][8..16].copy_from_slice(&decrypted[0][0..8]);

    let key = GenericArray::from(GO_KEY2);
    let mut decryptor = cbc::Decryptor::<Aes128>::new(&key, &iv);
    let block1 = GenericArray::from(response_payload[0]);
    let block2 = GenericArray::from(response_payload[1]);
    let mut blocks = [block1, block2];
    decryptor.decrypt_blocks_mut(&mut blocks);
    let decrypted = blocks.as_slice();

    let mut packet = [0u8; 40];
    packet[0..8].copy_from_slice(&[0x20, 0x01, 0x00, 0x00, 0x82, 0x82, 0x82, 0x82]);
    packet[8..24].copy_from_slice(decrypted[0].as_slice());
    packet[24..40].copy_from_slice(decrypted[1].as_slice());
    Ok(packet)
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
        info!("secret1 not found");
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
        info!("secret2 not found");
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
        let mut ctx = cbc::Encryptor::<Aes128>::new(&GenericArray::from(key.unwrap()), &GenericArray::from([0u8; 16]));
        let mut block = GenericArray::from(*plain_bytes);
        ctx.encrypt_block_mut(&mut block);
        encrypted.copy_from_slice(block.as_slice());
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
    (0xFFu16 - (sh & 0xffu16)) as u8
}

fn build_packet(code: u8, packet: &[u8]) -> ([u8;64], usize) {
    let mut full_packet = [0u8; 64];
    full_packet[0] = 0xA5;
    full_packet[1] = (packet.len() + 2) as u8;
    full_packet[2] = code;
    full_packet[3..packet.len()+3].copy_from_slice(packet);
    full_packet[packet.len() + 3] = checksum(&full_packet[0..packet.len()+3]);
    (full_packet, packet.len()+4)
}

fn fmt_packet(packet: &[u8], size: usize) -> heapless::String<512> {
    let mut msg = heapless::String::<512>::new();
    let _ = ufmt::uwrite!(msg, "[");
    for i in 0..size {
        let byte = packet[i];
        if i == size-1 {
            let _ = ufmt::uwrite!(msg, "0x{:02X}", byte);
        }
        else
        {
            let _ = ufmt::uwrite!(msg, "0x{:02X}, " byte);
        }
    }
    let _ = ufmt::uwrite!(msg, "]");
    msg
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
    CmdAuthGo = 0x90,
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
        let _ = embedded_logger::StdLogger::init();
        let challenge: [u8; 13] = [0x5A, 0x0B, 0x80, 0xD9, 0x8E, 0x35, 0xF3, 0x8F, 0x2B, 0x8C, 0x6D, 0x8F, 0x49];
        let expected_response: [u8; 20] = [
            0xA5, 0x12, 0x06, 0x83, 0x32, 0x32, 0xDE, 0xF3, 0x25, 0xA2,
            0x7C, 0x1A, 0xC9, 0x21, 0x7A, 0xE9, 0x8F, 0xBE, 0x22, 0x71,
        ];

        let code: u8 = ResponseType::Ack as u8;

        let challenge_version = challenge[3];
        let ch = &challenge[4..];
        if let Ok((packet, ch1b)) = cmdauth1(challenge_version, ch) {
            info!("{:x?}", ch1b);
            let send = build_packet(code, &packet);
            assert_eq!(send.0[19], expected_response[19]);

            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet(&send.0, send.1).as_str());
            debug!("{}", msg.as_str());
            assert_eq!(expected_response, send.0[..send.1]);
        } else {
            let packet = [0xff; 8];
            let send = build_packet(code, &packet);
            assert_eq!(expected_response, send.0[..send.1]);
        }
    }

    #[test]
    fn test_challenge_response_cmdauth2() {
        let _ = embedded_logger::StdLogger::init();
        let challenge: [u8; 12] = [0x5A, 0x0A, 0x81, 0x13, 0xF1, 0x06, 0x0B, 0x97, 0x9E, 0x9F, 0xF9, 0x38];
        let expected_response: [u8;20] = [
            0xA5, 0x12, 0x06, 0xBA, 0x54, 0x76, 0x57, 0x8E, 0xAF, 0x4E,
            0x8F, 0xAD, 0xF2, 0xA3, 0x55, 0xDA, 0x10, 0xC2, 0x1D, 0xED,
        ];


        let code: u8 = ResponseType::Ack as u8;
        let challenge_version = 0xD9;

        let challenge1b: [u8; 16] = [0x1A, 0xC9, 0x21, 0x7A, 0xE9, 0x8F, 0xBE, 0x22, 0x54, 0x0a, 0x8c, 0xbb, 0xc1, 0xac, 0xf7, 0xfa];

        if let Ok(packet) = cmdauth2(challenge_version, &challenge, &challenge1b) {
            let send = build_packet(code, &packet);
            assert_eq!(send.0[19], expected_response[19]);

            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet((&send.0), send.1).as_str());
            debug!("{}", msg.as_str());

            assert_eq!(expected_response, send.0[..send.1]);
        } else {
            assert!(false, "cmdauth2 returned err");
        }
    }

    #[test]
    fn test_challenge_response_cmdauth1_go() {
        let _ = embedded_logger::StdLogger::init();
        let challenge: [u8; 13] = [0x5A, 0x0B, 0x80, 0xB3, 0x38, 0xCF, 0x3D, 0xCD, 0x8E, 0x17, 0x1E, 0x03, 0x90];
        let expected_response: [u8; 20] = [
            0xA5, 0x12, 0x06, 0x1C, 0x76, 0x41, 0xAA, 0xB1, 0x43, 0x8A, 0xF5, 0x0D, 0xF8, 0xF8, 0x84, 0x95, 0x45, 0x84, 0x3A, 0x39
        ];

        let code: u8 = ResponseType::Ack as u8;

        let challenge_version = challenge[3];
        let ch = &challenge[4..];
        if let Ok((packet, ch1b)) = cmdauth1(challenge_version, ch) {
            debug!("ch1b: {:x?}", ch1b);
            let send = build_packet(code, &packet);
            //assert_eq!(send.0[19], expected_response[19]);

            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet((&send.0), send.1).as_str());
            debug!("{}", msg.as_str());
            assert_eq!(expected_response, send.0[..send.1]);
        } else {
            let packet = [0xffu8; 8];
            let send = build_packet(code, &packet);
            assert_eq!(expected_response, send.0[..send.1]);
        }
    }

    #[test]
    fn test_challenge_response_cmdauth2_go() {
        let _ = embedded_logger::StdLogger::init();
        let challenge: [u8; 12] = [0x5A, 0x0A, 0x81, 0xF0, 0x78, 0xB3, 0x21, 0xBD, 0x0A, 0x24, 0x16, 0xDD];
        let expected_response: [u8;20] = [
            0xA5, 0x12, 0x06, 0x8C, 0x39, 0xD6, 0x17, 0xD3, 0xD4, 0xF8,
            0x95, 0xB8, 0x88, 0x8A, 0x13, 0xD2, 0x7E, 0x73, 0xB1, 0x0B,
        ];


        let code: u8 = ResponseType::Ack as u8;
        let challenge_version = 0xB3;


        let challenge1b = [0x0d, 0xf8, 0xf8, 0x84, 0x95, 0x45, 0x84, 0x3a,
                           0x4d, 0x84, 0x7f, 0x54, 0x7a, 0xd6, 0x2d, 0x77];
        if let Ok(packet) = cmdauth2(challenge_version, &challenge, &challenge1b) {
            let send = build_packet(code, &packet);
            //assert_eq!(send.0[19], expected_response[19]);

            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet((&send.0), send.1).as_str());
            debug!("{}", msg.as_str());

            assert_eq!(expected_response, send.0[..send.1]);
        } else {
            assert!(false, "cmdauth2 returned err");
        }
    }


    #[test]
    fn test_challenge_response_cmdauthgo() {
        let _ = embedded_logger::StdLogger::init();
        let challenge = [0x5A, 0x2A, 0x90, 0x20, 0x10, 0x00, 0x06, 0x82, 0x82, 0x82, 0x82, 0xCB, 0xA3, 0xDB, 0xAC, 0x00, 0xDF, 0x26, 0xF8, 0xDD, 0x5B, 0x0D, 0xAC, 0x91, 0x9A, 0xCF, 0x0B, 0x63, 0x26, 0x06, 0x18, 0xE6, 0x30, 0x4F, 0xDF, 0xE1, 0x6C, 0xEE, 0xA5, 0x16, 0x4E, 0x94, 0x15, 0xED];
        let expected_response = [0xA5, 0x2A, 0x06, 0x20, 0x01, 0x00, 0x00, 0x82, 0x82, 0x82, 0x82, 0x82, 0x62, 0xDA, 0xD6, 0x79, 0x3C, 0x82, 0x92, 0x50, 0xEB, 0xC8, 0x86, 0x37, 0x23, 0x49,0x49,0xF5, 0xE6, 0x97, 0xC2, 0xF0, 0x76, 0x05, 0x73, 0xD7, 0x59, 0x2D, 0xC6, 0xE5, 0x27,0x5F,0x6D,0x22];
        let code = ResponseType::Ack as u8;
        let screq = &challenge[3..];
    
        if let Ok(packet) = cmdauthgo(&screq) {
            let send = build_packet(code, &packet);
            let mut msg = heapless::String::<512>::new();
            let _ = ufmt::uwrite!(msg, "Sending packet: ");
            let _ = msg.write_str(fmt_packet((&send.0), send.1).as_str());
            debug!("{}", msg.as_str());

            assert_eq!(expected_response, send.0[..send.1]);
        } else {
            assert!(false, "cmdauthgo returned err");
        }
    }

    #[test]
    fn test_cmdauth1_invalid_version() {
        assert!(cmdauth1(0x55, &[0x00; 1]).is_err());
    }
}
