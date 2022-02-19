#![no_std]

pub extern crate rp2040_hal as hal;

#[cfg(feature = "rt")]
extern crate cortex_m_rt;
#[cfg(feature = "rt")]
pub use cortex_m_rt::entry;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[cfg(feature = "boot2")]
#[link_section = ".boot2"]
#[no_mangle]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_GD25Q64CS;

pub use hal::pac;

hal::bsp_pins!(
    Gpio0 {
        name: sda,
        aliases: { FunctionI2C: Sda }
    },
    Gpio1 {
        name: scl,
        aliases: { FunctionI2C: Scl }
    },
    Gpio2 { name: d5 },
    Gpio3 { name: d6 },
    Gpio4 {
        name: txd,
        aliases: { FunctionUart: UartTx }
    },
    Gpio5 {
        name: rxd,
        aliases: { FunctionUart: UartRx }
    },
    Gpio6 { name: d9 },
    Gpio7 { name: d10 },
    Gpio8 { name: d11 },
    Gpio9 { name: d12 },
    Gpio10 { name: d13 },
    Gpio11 { name: neopixel },
    Gpio12 { name: led },
    Gpio13 { name: wifi_boot },
    Gpio16 { name: tx },
    Gpio17 { name: rx },
    Gpio19 { name: w_rst },
    Gpio22 { name: sck },
    Gpio23 { name: sdo },
    Gpio24 { name: sdi },
    Gpio25 { name: a4 },
    Gpio26 { name: a0 },
    Gpio27 { name: a1 },
    Gpio28 { name: a2 },
    Gpio29 { name: a3 },
);

pub const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;
