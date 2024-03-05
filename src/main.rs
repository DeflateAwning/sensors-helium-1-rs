#![no_std]
#![no_main]

use core::f32::NAN;

use esp32_hal::{clock::ClockControl, peripherals::Peripherals, IO, prelude::*, Delay};
use esp_backtrace as _;
use esp_println::println;
use esp32_hal::i2c::I2C;

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

    let mut loop_count: u64 = 0;
    println!("Booting...");

    loop {
        println!("Starting I2C scan:\n");
        scan_i2c_devices(&mut i2c_port).unwrap();
        println!("Done I2C scan.\n");

        let light_val: f32 = read_light_sensor_lx(&mut i2c_port, &mut light_sensor).unwrap_or(NAN);
        let temp_celsius: f32 = read_temperature_c(&mut i2c_port, LM75_ADDRESS).unwrap_or(-100.0);

        println!("Loop {loop_count}, light: {light_val} lx, temp: {temp_celsius}...",
            loop_count=loop_count,
            light_val=light_val
        );
        delay.delay_ms(500u32);

        loop_count += 1;
    }
}
