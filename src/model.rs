use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{fs, path::Path};

use crate::error::{self, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub display: Display,
    pub weather: Weather,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Display {
    pub gpio_dev: String,
    pub data_pin: u32,
    pub cs_pin: u32,
    pub clk_pin: u32,
    pub number_of_matrices: u32,
    pub brightness: u8,
    pub slim: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Weather {
    pub display_interval_sec: u8,
    pub humidity_on_display_msec: u64,
    pub temperature_on_display_msec: u64,
    pub sensor: WeatherSensor,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherSensor {
    pub gpio_dev: String,
}

impl Config {
    #[inline]
    pub fn new() -> Self {
        Config {
            display: Display {
                gpio_dev: "/dev/gpiochip0".to_string(),
                data_pin: 85, // mosi      -> DIN      (1)  on MAX7221
                cs_pin: 83,   // cs        -> LOAD(CS) (12) on MAX7221
                clk_pin: 84,  // clk(sck)  -> CLK      (13) on MAX7221
                number_of_matrices: 4,
                brightness: 0x0F, // max
                slim: false,
            },
            weather: Weather {
                display_interval_sec: 20,
                humidity_on_display_msec: 1000,
                temperature_on_display_msec: 1500,
                sensor: WeatherSensor { gpio_dev: "/dev/gpiochip0".to_string() },
            },
        }
    }

    pub fn to_yaml(&self, output: &Path) -> Result<(), Error> {
        let serialized = serde_yaml::to_string(self).context(error::ConfigSnafu)?;
        fs::write(output, &serialized).context(error::ConfigIoSnafu)?;

        Ok(())
    }

    pub fn from_yaml(yaml: &Path) -> Result<Self, Error> {
        let file = fs::File::open(yaml).context(error::ConfigIoSnafu)?;
        let config = serde_yaml::from_reader(file).context(error::ConfigSnafu)?;

        Ok(config)
    }
}
