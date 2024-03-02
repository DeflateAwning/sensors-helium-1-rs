#![no_std]
#![no_main]

use esp32_hal::{clock::ClockControl, peripherals::Peripherals, IO, prelude::*, Delay};
use esp_backtrace as _;
use esp_println::println;
use embedded_sensors::bh1750::{Bh1750, config::MeasurementMode};
use esp32_hal::i2c::I2C;
// use lm75::{Lm75, Address};

const BH1750_ADDRESS: u8 = 0x23;

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
        io.pins.gpio21,
        io.pins.gpio22,
        100u32.kHz(),
        &clocks,
    );
    let mut light_sensor = Bh1750::new(BH1750_ADDRESS, &mut i2c_port).unwrap();


    // light_sensor.set_measurement_mode(&mut i2c_port, MeasurementMode::OneTimeHighResolution2).unwrap();
    light_sensor.set_measurement_mode(&mut i2c_port, MeasurementMode::ContinuouslyHighResolution2).unwrap();

    // let temperature_chip_dev = I2C::new(peripherals.I2C1, i2c_sda, i2c_scl, 100u32.kHz(), &clocks);
    // let address = Address::default();
    // let mut sensor = Lm75::new(temperature_chip_dev, address);
    // let temp_celsius = sensor.read_temperature().unwrap();
    // println!("Temperature: {}ºC", temp_celsius);


    let mut loop_count: u16 = 1.212e2 as u16;

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

        println!("Loop {loop_count}, light: {light_val}...",
            loop_count=loop_count,
            light_val=light_val
        );
        delay.delay_ms(500u32);

        loop_count += 1;
    }
}
