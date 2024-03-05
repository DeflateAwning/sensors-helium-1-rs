use embedded_hal::blocking::i2c::Read;
use esp_println::println;

pub fn scan_i2c_devices<I2C>(i2c_port: &mut I2C) -> Result<(), I2C::Error>
where
    I2C: Read,
{
    // Start Scan at Address 1 going up to 127
    for addr in 1..=127 {
        // println!("Scanning Address {}", addr as u8);

        // Scan Address
        let res = i2c_port.read(addr as u8, &mut [0]);

        // Check and Print Result
        match res {
            Ok(_) => println!("Device Found at Address d{}", addr as u8),
            Err(_) => (), //println!("No Device Found"),
        }
    }

    Ok(())
}
