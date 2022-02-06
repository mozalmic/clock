use crate::aht10::AHT10;
use linux_embedded_hal::I2cdev;
use snafu::ResultExt;

use crate::{
    display,
    error::{self, Error},
};

pub(crate) enum WeatherType {
    Humidity,
    Temperature,
}

pub(crate) struct Weather {
    humidity: u8,
    temperature: f32,
}

pub(crate) fn draw(
    display: &mut display::LinearMatrixDisplay,
    sensor: &mut AHT10<I2cdev>,
    weather_type: WeatherType,
    slim: bool,
) -> Result<(), Error> {
    let weather = measure(sensor)?;

    let (humidity, percent, dot, celsius, nums) = if slim {
        (
            display::SLIM_HUMIDITY,
            display::SLIM_PERCENT,
            display::SLIM_DOT,
            display::SLIM_CELSIUS,
            display::SLIM_NUMS,
        )
    } else {
        (
            display::HUMIDITY,
            display::PERCENT,
            display::DOT,
            display::CELSIUS,
            display::NUMS,
        )
    };

    let h1 = nums[(weather.humidity / 10) as usize];
    let h2 = nums[(weather.humidity % 10) as usize];

    let temperature = (weather.temperature * 10.0) as u32;
    let t1 = nums[(temperature / 100) as usize];
    let t2 = nums[(temperature % 100 / 10) as usize];
    let t3 = nums[(temperature % 10) as usize];

    match weather_type {
        WeatherType::Humidity => display.draw(|x, y| {
            if x >= 1 && x <= 7 {
                humidity[y] & (1 << (7 - x))
            } else if x >= 10 && x <= 15 {
                h1[y] & (1 << (15 - x))
            } else if x >= 17 && x <= 22 {
                h2[y] & (1 << (22 - x))
            } else if x >= 24 && x <= 29 {
                percent[y] & (1 << (29 - x))
            } else {
                0
            }
        })?,
        WeatherType::Temperature => display.draw(|x, y| {
            if x >= 1 && x <= 6 {
                t1[y] & (1 << (6 - x))
            } else if x >= 8 && x <= 13 {
                t2[y] & (1 << (13 - x))
            } else if x >= 15 && x <= 16 {
                dot[y] & (1 << (16 - x))
            } else if x >= 18 && x <= 23 {
                t3[y] & (1 << (23 - x))
            } else if x >= 25 && x <= 30 {
                celsius[y] & (1 << (30 - x))
            } else {
                0
            }
        })?,
    }

    Ok(())
}

fn measure(sensor: &mut AHT10<I2cdev>) -> Result<Weather, Error> {
    let (t, h) = sensor.measure().context(error::SensorSnafu)?;
    let weather = Weather { humidity: h as u8, temperature: t };
    Ok(weather)
}
