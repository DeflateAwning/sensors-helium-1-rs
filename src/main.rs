#![no_std]
#![no_main]

use esp32_hal::{clock::ClockControl, peripherals::Peripherals, IO, prelude::*, Delay};
use esp_backtrace as _;
use esp_println::println;
use embedded_sensors::bh1750::{Bh1750, config::MeasurementMode};
use esp32_hal::i2c::I2C;
use embedded_hal::blocking::i2c::Read; // for read_temperature_c
use lm75::Lm75;

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

    // SDA=GPIO21, SCL=GPIO22
    let mut i2c_port = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21, // SDA
        io.pins.gpio22, // SCL
        100u32.kHz(),
        &clocks,
    );

    // setup the light sensor
    let mut light_sensor = Bh1750::new(BH1750_ADDRESS, &mut i2c_port).unwrap();
    light_sensor.set_measurement_mode(&mut i2c_port, MeasurementMode::ContinuouslyHighResolution2).unwrap();

    let mut sensor = Lm75::new(&mut i2c_port, LM75_ADDRESS);
    loop {
        let temp_celsius = sensor.read_temperature().unwrap();
        println!("Temperature: {}C", temp_celsius);
    }

    let mut loop_count: u64 = 0;

    println!("Booting...");
    loop {
        // Start Scan at Address 1 going up to 127
        // for addr in 1..=127 {
        //     println!("Scanning Address {}", addr as u8);

        //     // Scan Address
        //     let res = i2c_port.read(addr as u8, &mut [0]);

        //     // Check and Print Result
        //     match res {
        //         Ok(_) => println!("Device Found at Address {}", addr as u8),
        //         Err(_) => (), //println!("No Device Found"),
        //     }
        // }

        light_sensor.read(&mut i2c_port).unwrap();
        let light_val: f32 = light_sensor.light_level();

        let temp_celsius = read_temperature_c(&mut i2c_port, LM75_ADDRESS).unwrap();

        println!("Loop {loop_count}, light: {light_val} lx, temp: {temp_celsius}...",
            loop_count=loop_count,
            light_val=light_val
        );
        delay.delay_ms(500u32);

        loop_count += 1;
    }
}

pub fn read_temperature_c<T>(i2c: &mut T, address: u8) -> Result<f32, T::Error>
where
    T: Read,
{
    let mut buffer: [u8; 2] = [0u8; 2];
    match i2c.read(address, &mut buffer) {
        Ok(()) => {
            let mut ret: f32 = 0.0;
            
            // check sign bit
            if (buffer[0] & 0b1000_0000) != 0 {
                ret = -128.0;
            }
            
            // Source: https://github.com/elhep/stm_system_board_firmware/blob/a1b5af04b61ece350be88540e9c8bc34c01abd28/src/hardware/lm75a.rs#L4
            Ok(ret + (buffer[0] & 0b0111_1111) as f32 + 0.5 * (buffer[0] & 0b1000_0000) as f32)
        },
        Err(e) => Err(e),
    }
}
