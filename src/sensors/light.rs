use esp32_hal::i2c::Error;
use embedded_sensors::bh1750::{Bh1750, config::MeasurementMode};
use embedded_hal::blocking::i2c::{Read, Write};

pub fn setup_light_sensor<T>(i2c: &mut T, address: u8) -> Result<Bh1750<T>, Error>
where
	T: Write<Error = Error>,
	T: Read<Error = Error>,
{
    let mut light_sensor = Bh1750::new(address, i2c).unwrap();
    light_sensor.set_measurement_mode(i2c, MeasurementMode::ContinuouslyHighResolution2).unwrap();
    Ok(light_sensor)
}

pub fn read_light_sensor_lx<T>(i2c: &mut T, light_sensor: &mut Bh1750<T>) -> Result<f32, Error>
where
	T: Write<Error = Error>,
	T: Read<Error = Error>,
{
    light_sensor.read(i2c).unwrap();
    Ok(light_sensor.light_level())
}
