//! Rainbow effect color wheel using the onboard NeoPixel on an iLabs Challenger RP2040 Wifi board
//!
//! This flows smoothly through various colors on the onboard NeoPixel.
//! Uses the `ws2812_pio` driver to control the NeoPixel, which in turns uses the
//! RP2040's PIO block.
//!
//! Also connects to the WiFi AP configured in [ssid] with [password]
//! and sends UDP packets to the given IP address [ip_address].
#![no_std]
#![no_main]

use core::iter::once;
use cortex_m_rt::entry;
use embedded_hal::{digital::v2::OutputPin, timer::CountDown};
use embedded_time::duration::Extensions;
use ilabs_challenger_rp2040_wifi::{
    hal,
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        pac,
        pio::PIOExt,
        timer::Timer,
        watchdog::Watchdog,
        Sio,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};
use panic_halt as _;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use ws2812_pio::Ws2812;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut delay = timer.count_down();

    let mut led = pins.led.into_push_pull_output();

    // Configure the addressable LED
    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let mut ws = Ws2812::new(
        // The onboard NeoPixel is attached to GPIO pin #16 on the Feather RP2040.
        pins.neopixel.into_mode(),
        &mut pio,
        sm0,
        clocks.peripheral_clock.freq(),
        timer.count_down(),
    );

    let mut w_rst = pins.w_rst.into_push_pull_output();
    let mut w_mode = pins.wifi_boot.into_push_pull_output();

    let uart_pins = (
        pins.txd.into_mode::<hal::gpio::FunctionUart>(),
        pins.rxd.into_mode::<hal::gpio::FunctionUart>(),
    );
    let uart = hal::uart::UartPeripheral::new(pac.UART1, uart_pins, &mut pac.RESETS)
        .enable(
            hal::uart::common_configs::_115200_8_N_1,
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let mut buf = [0u8; 255];
    uart.read_raw(&mut buf);
    uart.read_raw(&mut buf);
    uart.read_raw(&mut buf);
    uart.write_full_blocking(b"AT+CWSTATE?\r\n");
    led.set_high().unwrap();

    delay.start(25.milliseconds());
    let _ = nb::block!(delay.wait());
    let mut c = 0u8;
    match uart.read_raw(&mut buf) {
        Ok(count) => {
            for _ in 0..count {
                c = count as u8;
                led.set_high().unwrap();
                delay.start(25.milliseconds());
                let _ = nb::block!(delay.wait());
                led.set_low().unwrap();
                delay.start(25.milliseconds());
                let _ = nb::block!(delay.wait());
            }
        }
        Err(e) => {
            c = match e {
                Overrun => 2,
                Break => 3,
                Parity => 4,
                Framing => 5,
            };
            led.set_low().unwrap();
        }
    }
    let mut step = 1;

    // Infinite colour wheel loop
    let mut buf = [0u8; 255];
    let mut color_wheel: u8 = 128;

    let hostname = b"Challenger";
    let ssid = b"";
    let pwd = b"";
    let ip_address = b"";

    loop {
        ws.write(brightness(once(wheel(color_wheel)), 3)).unwrap();
        color_wheel = color_wheel.wrapping_add(1);
        if color_wheel % 32 == 0 {
            match step {
                1 => {
                    w_rst.set_low().unwrap();
                    w_mode.set_high().unwrap();
                    w_rst.set_high().unwrap();
                    step += 1;
                }
                2 => {
                    uart.write_full_blocking(b"AT+CWHOSTNAME=\"");
                    uart.write_full_blocking(hostname);
                    uart.write_full_blocking(b"\"\r\n");
                    uart.write_full_blocking(b"AT+CWMODE=3\r\n");
                    step += 1;
                }
                3 => {
                    uart.write_full_blocking(b"AT+CWJAP=\"");
                    uart.write_full_blocking(ssid);
                    uart.write_full_blocking(b"\",\"");
                    uart.write_full_blocking(pwd);
                    uart.write_full_blocking(b"\"\r\n");
                    step += 1;
                }
                4 => {
                    uart.write_full_blocking(b"AT+CIFSR\r\n");
                    step += 1;
                }
                5 => {
                    uart.write_full_blocking(b"AT+CIPSTART=\"UDP\",\"");
                    uart.write_full_blocking(ip_address);
                    uart.write_full_blocking(b"\",8080,1112,2\r\n");
                    step += 1;
                }
                _ => {
                    match step % 2 == 0 {
                        true => {
                            uart.write_full_blocking(b"AT+CIPSEND=10\r\n");
                        }
                        false => {
                            uart.write_full_blocking(b"UDPtest ");
                            uart.write_full_blocking(&[b'0' + c]);
                            uart.write_full_blocking(b"\n");
                            uart.write_full_blocking(b"AT+CIPCLOSE");
                            // c = (c + 1) % 10;
                        }
                    }
                    step += 1;
                }
            }
        }

        delay.start(25.milliseconds());
        let _ = nb::block!(delay.wait());
    }
}

/// Convert a number from `0..=255` to an RGB color triplet.
///
/// The colours are a transition from red, to green, to blue and back to red.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        // No green in this sector - red and blue only
        (255 - (wheel_pos * 3), 0, wheel_pos * 3).into()
    } else if wheel_pos < 170 {
        // No red in this sector - green and blue only
        wheel_pos -= 85;
        (0, wheel_pos * 3, 255 - (wheel_pos * 3)).into()
    } else {
        // No blue in this sector - red and green only
        wheel_pos -= 170;
        (wheel_pos * 3, 255 - (wheel_pos * 3), 0).into()
    }
}
