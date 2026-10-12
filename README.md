# Baryonsweeper-rs
A portable, microcontroller-based [Baryonsweeper](https://www.psdevwiki.com/psp/Psp-3000_Jigkick) implemented in [Embedded Rust](https://rust-embedded.org/)

## Portability
This project abstracts away the Baryonsweeper UART protocol, and runs on multiple different Development Boards including
- Raspberry Pi Pico
- Adafruit ItsyBitsy M0
- Adafruit Metro M4
- I also planned a target for embedded-linux SBCs like a full Raspberry Pi Zero/1/2/3 but never finished implementing it.

It should be simple to port to other boards in the embedded-rust ecosystem.

## Raspberry Pi Pico Example

### Building the project
Navigate to the directory for the desired target platform (ie `cd baryonsweeper-rp2040`) and run `cargo build --release` to produce an ELF file

### Converting to UF2 for the pico bootloader
- cargo install [elf2uf2-rs](https://crates.io/crates/elf2uf2-rs)
- `elf2uf2-rs target/thumbv6m-none-eabi/release/baryonsweeper-rp2040 target/thumbv6m-none-eabi/release/baryonsweeper-rp2040.uf2`
- The output file can then be copied to any pico in bootloader mode to program it.
(Press bootsel while plugging in usb to get into bootloader mode, then drag the file to the usb drive the bootloader presents)
- Alternatively, you can run `elf2uf2-rs -d -s target/thumbv6m-none-eabi/release/baryonsweeper-rp2040` to program directly and attach to the usb serial console.

### Wiring
The PSP Battery interface presents a K-Line UART (single wire) on the middle pin.

Connect a 1N4148 diode between GP0 and GP1 of the Pico, with the line on the diode towards GP0.

Connect the wire side of a test clip to the GP1 side of the diode, and connect the other end of the test clip to the PSP's middle battery pin.

Connect 2 more test clips to GND and 3v3 on the pico, and wire to pins 1 and 3 of the PSP battery connector respectively.

Refer to [psdevwiki](https://www.psdevwiki.com/psp/Psp-3000_Jigkick) for a detailed wiring diagram with slightly different hardware.

### Reading USB Serial Logs
This project outputs logs over the USB TTY ACM Interface on most boards, or sometimes another free UART on the microcontroller.
To read the USB logs, connect PuTTY to the associated COM port on Windows, or minicom/picocom to the `ttyACMx`on linux.
The `elf2uf2-rs -d -s` command above should also do this more automagically.

## Running tests
```sh
cd baryonsweeper
cargo test --features=std,test --release -- --no-capture
```
