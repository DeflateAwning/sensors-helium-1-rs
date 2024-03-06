#![no_std]
#![no_main]

#![feature(alloc_error_handler)]

use arrayvec::{ArrayVec};
use core::f32::NAN;
use esp32_hal::prelude::nb::block;
use esp32_hal::uart::TxRxPins;
use esp32_hal::{clock::ClockControl, peripherals::Peripherals, IO, prelude::*, Delay};
use esp_backtrace as _;
use esp_println::{println, print};
use esp32_hal::i2c::I2C;
use esp32_hal::uart::{config::Config as UartConfig, Uart};
use core::fmt::Write;

// from this project
use sensors::{
    temperature::read_temperature_c,
    light::{setup_light_sensor, read_light_sensor_lx},
    i2c_scanner::scan_i2c_devices};
mod sensors;


const BH1750_ADDRESS: u8 = 0x23;
const LM75_ADDRESS: u8 = 0x48;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    // let i2c_sda = peripherals.IO_MUX.gpio21();
    // let i2c_scl = peripherals.IO_MUX.gpio22();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut onboard_led = io.pins.gpio4.into_push_pull_output();

    // SDA=GPIO21, SCL=GPIO22
    let mut i2c_port = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21, // SDA
        io.pins.gpio22, // SCL
        100u32.kHz(),
        &clocks,
    );

    // setup the light sensor
    let mut light_sensor = setup_light_sensor(&mut i2c_port, BH1750_ADDRESS).unwrap();

    // setup the LoRa UART
    let mut lora_uart = Uart::new_with_config(
        peripherals.UART1,
        UartConfig {
            baudrate: 9600u32,
            ..Default::default() // 8N1
        },
        Some(TxRxPins::new_tx_rx(
            io.pins.gpio17.into_push_pull_output(), // TX
            io.pins.gpio16.into_floating_input(), // RX
        )),
        &clocks);

    let mut loop_count: u64 = 0;
    println!("Booting...");

    loop {
        onboard_led.toggle().unwrap();

        println!("Starting I2C scan:\n");
        scan_i2c_devices(&mut i2c_port).unwrap();
        println!("Done I2C scan.\n");

        let light_val: f32 = read_light_sensor_lx(&mut i2c_port, &mut light_sensor).unwrap_or(NAN);
        let temp_celsius: f32 = read_temperature_c(&mut i2c_port, LM75_ADDRESS).unwrap_or(-100.0);

        writeln!(&mut lora_uart, "AT").unwrap();
        println!("MCU->LoRa: AT");

        // wait a sec to ensure the response is sent
        delay.delay_ms(100u32);

        let mut rx_bytes = ArrayVec::<u8, 250>::new();
        loop {
            match lora_uart.read() {
                Ok(rx_byte) => {
                    // TODO: deal with out-of-space error here
                    rx_bytes.push(rx_byte);
                },
                Err(nb::Error::WouldBlock) => {
                    // not an interesting error; just means we're out of incoming bytes
                    break;
                }
                Err(e) => {
                    // Handle other errors: print it out and break
                    println!("LoRa->MCU: [ERROR]: {e:?}");
                    break;
                }
            }
        }
        
        println!("LoRa->MCU: {rx_bytes:?}");
        let rec_str = core::str::from_utf8(&rx_bytes).unwrap();
        println!("LoRa->MCU: {rec_str}");

        println!("Loop {loop_count}, light: {light_val} lx, temp: {temp_celsius}, rx_count: {rec_len:?}...",
            loop_count = loop_count,
            light_val = light_val,
            rec_len = rx_bytes.len(),
        );
        delay.delay_ms(500u32);

        loop_count += 1;
    }
}
