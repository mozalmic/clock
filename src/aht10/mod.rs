pub mod error;

use embedded_hal::blocking::i2c::{Read, Write};
use error::Error;
use snafu::ResultExt;

const I2C_ADDRESS: u8 = 0x38; // AHT10 has it's own static address

const CMD_INIT: [u8; 3] = [0b11100001, 0b00001000, 0];
const CMD_MEASURE: [u8; 3] = [0b10101100, 0b00110011, 0];

bitflags! {
    struct StatusFlags: u8 {
        const BUSY = (1 << 7); // 1 - device is busy
        const MODE = ((1 << 6) | (1 << 5)); // 00 - NOR, 01 - CYC, 1x - CMD
        const CALIBRATION = (1 << 3); // 1 - calibrated
    }
}

pub struct AHT10<I2C> {
    pub(crate) i2c: I2C,
    pub(crate) delay_ms: fn(u32) -> (),
}

impl<I2C, I2CError> AHT10<I2C>
where
    I2CError: std::error::Error,
    I2C: Read<Error = I2CError> + Write<Error = I2CError>,
{
    pub fn init(&mut self) -> Result<(), Error<I2CError>> {
        self.i2c.write(I2C_ADDRESS, &CMD_INIT).context(error::InitSnafu)?;
        (self.delay_ms)(300);
        Ok(())
    }

    /// Measures Temperature and Relative Humidity
    pub fn measure(&mut self) -> Result<(f32, f32), Error<I2CError>> {
        self.i2c.write(I2C_ADDRESS, &CMD_MEASURE).context(error::MeasureSnafu)?;
        (self.delay_ms)(100);

        let buf: &mut [u8; 6] = &mut [0; 6];
        self.i2c.read(I2C_ADDRESS, buf).context(error::MeasureSnafu)?;
        let status = StatusFlags { bits: buf[0] };
        if !status.contains(StatusFlags::CALIBRATION) {
            return Err(Error::UncalibratedError {});
        }

        let hum = ((buf[1] as u32) << 12) | ((buf[2] as u32) << 4) | ((buf[3] as u32) >> 4);
        // to relative (0-100%)
        let hum = 100.0 * (hum as f32) / ((1 << 20) as f32);

        let temp = (((buf[3] as u32) & 0x0f) << 16) | ((buf[4] as u32) << 8) | (buf[5] as u32);
        // to celsius
        let temp = (200.0 * (temp as f32) / ((1 << 20) as f32)) - 50.0;

        Ok((temp, hum))
    }
}
