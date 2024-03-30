#![no_std]
#![no_main]

#![feature(alloc_error_handler)]

use arrayvec::{ArrayVec};
use core::f32::NAN;
use esp32_hal::prelude::nb::block;
use esp32_hal::uart::TxRxPins;
use esp32_hal::{clock::ClockControl, peripherals::Peripherals, IO, prelude::*, Delay};
use esp32_hal::{UartTx, UartRx};

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
use lora_e5::{
    LoRaE5, Command, Reply
};
mod lora_e5;


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
    let lora_uart = Uart::new_with_config(
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

    // setup the LoRaE5 driver, using the UART
    let (lora_uart_tx, lora_uart_rx) = lora_uart.split();
    let mut lora_e5 = LoRaE5::new(lora_uart_tx, lora_uart_rx);

    let mut loop_count: u64 = 0;
    println!("Booting...");

    loop {
        onboard_led.toggle().unwrap();

        println!("Starting I2C scan:\n");
        scan_i2c_devices(&mut i2c_port).unwrap();
        println!("Done I2C scan.\n");

        let light_val: f32 = read_light_sensor_lx(&mut i2c_port, &mut light_sensor).unwrap_or(NAN);
        let temp_celsius: f32 = read_temperature_c(&mut i2c_port, LM75_ADDRESS).unwrap_or(-100.0);

        // println!("Sending AT+ID command to LoRaE5...\n");
        // match lora_e5.send_command(Command::AT_ID_READ) {
        //     Ok(reply) => {
        //         match reply {
        //             Reply::AT_ID_READ(at_id_read_result) => {
        //                 println!("Received AT+ID reply from LoRaE5: {:?}\n", at_id_read_result);
        //             }
        //             _ => {
        //                 println!("Received unexpected reply from LoRaE5.\n");
        //             }
        //         }
        //     }
        //     Err(err) => {
        //         println!("Error sending AT+ID command to LoRaE5: {:?}\n", err);
        //     }
        // }
        // println!("Done sending AT+ID command to LoRaE5.\n");

        println!("Sending AT command to LoRaE5...\n");
        match lora_e5.send_command(Command::CheckAlive) {
            Ok(reply) => {
                match reply {
                    Reply::CheckAlive(cmd_result) => {
                        println!("Received AT reply from LoRaE5: {:?}\n", cmd_result);
                    }
                    _ => {
                        println!("Received unexpected reply from LoRaE5.\n");
                    }
                }
            }
            Err(err) => {
                println!("Error sending AT command to LoRaE5: {:?}\n", err);
            }
        }
        println!("Done sending AT command to LoRaE5.\n");





        println!("Loop {loop_count}, light: {light_val} lx, temp: {temp_celsius}",
            loop_count = loop_count,
            light_val = light_val
        );
        delay.delay_ms(500u32);

        loop_count += 1;
    }
}
