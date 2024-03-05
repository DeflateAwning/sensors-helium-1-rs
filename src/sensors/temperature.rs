

use embedded_hal::blocking::i2c::Read;

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
