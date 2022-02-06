use clock_macro::SnafuDebug;
use snafu::Snafu;

use crate::aht10;

#[derive(Snafu, SnafuDebug)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Cannot (de)serialize config to yaml."))]
    ConfigError { source: serde_yaml::Error },
    #[snafu(display("Cannot read/write config to/from disk."))]
    ConfigIoError { source: std::io::Error },

    #[snafu(display("Limit of 16 matrixes exceeded, used {}.", number))]
    Max7219NumberError { number: u32 },
    #[snafu(display("MAX7219 connection error."))]
    Max7219Error { source: gpio_cdev::Error },
    #[snafu(display("MAX7219 data error."))]
    Max7219DataError { source: DataError },

    #[snafu(display("I2C connection error."))]
    I2CError { source: i2cdev::linux::LinuxI2CError },
    #[snafu(display("I2C communication error."))]
    SensorError {
        source: aht10::error::Error<i2cdev::linux::LinuxI2CError>,
    },
}

// WRAPPERS for max7219::DataError
#[derive(Debug)]
pub struct DataError(pub max7219::DataError);

impl core::fmt::Display for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "({:?})", self.0)
    }
}

impl std::error::Error for DataError {}
