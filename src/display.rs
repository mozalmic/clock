use crate::error::{self, DataError, Error};
use embedded_hal::digital::v2::OutputPin;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use max7219::{connectors::PinConnector, DecodeMode, MAX7219};
use snafu::ResultExt;

pub struct LinearMatrixDisplay {
    screen: Max7219,
    number_of_matrices: usize,
}

impl LinearMatrixDisplay {
    /// Creates the MAX7219-based display constructed with N linear 8x8 led matrixes.
    /// It supports up to 16 led matrixes, works over SPI protocol on Linux devices only.
    /// It uses GPIO ports directly and does not require SPI module set up on Raspbery Pi.
    /// After initialization the display stays cleared and required brightness level is set.
    /// See your board documentation to get appropriate pin numbers (SPI port numbers).
    ///
    /// * `gpio_dev` the GPIO device ("/dev/gpiochip0" or similar)
    /// * `data_pin` SPI data port number
    /// * `clk_pin` SPI clock port number
    /// * `cs_pin` SPI chip select port number
    /// * `number_of_matrices` number of led matrices in your assembly.
    /// * `brightness` level between 0x00 to 0x0F
    pub fn new(
        gpio_dev: &str,
        data_pin: u32,
        cs_pin: u32,
        clk_pin: u32,
        number_of_matrices: u32,
        brightness: u8,
    ) -> Result<LinearMatrixDisplay, Error> {
        if number_of_matrices == 0 || number_of_matrices > 16 {
            return Err(Error::Max7219NumberError { number: number_of_matrices });
        }
        let number_of_matrices: usize = number_of_matrices.try_into().unwrap();

        let mut gpio = Chip::new(gpio_dev).context(error::Max7219Snafu)?;
        let data_pin = gpio
            .get_line(data_pin)
            .context(error::Max7219Snafu)?
            .request(LineRequestFlags::OUTPUT, 0, "spi-data-pin")
            .context(error::Max7219Snafu)?;
        let cs_pin = gpio
            .get_line(cs_pin)
            .context(error::Max7219Snafu)?
            .request(LineRequestFlags::OUTPUT, 0, "spi-cs-pin")
            .context(error::Max7219Snafu)?;
        let clk_pin = gpio
            .get_line(clk_pin)
            .context(error::Max7219Snafu)?
            .request(LineRequestFlags::OUTPUT, 0, "spi-clk-pin")
            .context(error::Max7219Snafu)?;

        let data_pin = Port(data_pin);
        let clk_pin = Port(clk_pin);
        let cs_pin = Port(cs_pin);

        let mut max7219 = MAX7219::from_pins(number_of_matrices, data_pin, cs_pin, clk_pin)
            .map_err(DataError)
            .context(error::Max7219DataSnafu)?;
        max7219.power_on().map_err(DataError).context(error::Max7219DataSnafu)?;
        for i in 0..number_of_matrices {
            // sets the DecodeMode to NoDecode which is necessary for displaying content on
            // the 8x8 matrix display. (Max7219 can also be used for 7 segment displays).
            max7219
                .set_decode_mode(i, DecodeMode::NoDecode)
                .map_err(DataError)
                .context(error::Max7219DataSnafu)?;
        }

        let mut display = LinearMatrixDisplay {
            screen: max7219,
            number_of_matrices: number_of_matrices,
        };

        display.clear()?;
        display.brightness(brightness)?;

        return Ok(display);
    }

    /// Sets display brightness
    /// * `intensity` - value between `0x00` and `0x0F`
    pub fn brightness(self: &mut Self, intensity: u8) -> Result<(), Error> {
        for i in 0..self.number_of_matrices {
            self.screen
                .set_intensity(i, intensity)
                .map_err(DataError)
                .context(error::Max7219DataSnafu)?;
        }
        Ok(())
    }

    /// Clears the display
    pub fn clear(self: &mut Self) -> Result<(), Error> {
        for i in 0..self.number_of_matrices {
            self.screen
                .clear_display(i)
                .map_err(DataError)
                .context(error::Max7219DataSnafu)?;
        }
        Ok(())
    }

    pub fn draw<F>(self: &mut Self, pixel: F) -> Result<(), Error>
    where
        F: Fn(usize, usize) -> u8,
    {
        let mut data: Vec<[u8; 8]> = vec![[0; 8]; self.number_of_matrices];

        for i in 0..self.number_of_matrices {
            for y in 0..=7 {
                let mut line = data[i][y];
                for x in 0..=7 {
                    if pixel(x + i * 8, y) > 0 {
                        line = line | (1 << (7 - x));
                    }
                }
                data[i][y] = line;
            }
        }

        for i in 0..self.number_of_matrices {
            self.screen
                .write_raw(i, &data[i])
                .map_err(DataError)
                .context(error::Max7219DataSnafu)?;
        }

        Ok(())
    }
}

type Max7219 = MAX7219<PinConnector<Port, Port, Port>>;

/// MAX7219 crate works with OutputPin trate, but `gpio_cdev::LineHandle` does not implement it.
/// `Port` structure wraps `LineHandle` and adds missing functionality
struct Port(LineHandle);
impl OutputPin for Port {
    type Error = gpio_cdev::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(0)
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_value(1)
    }
}

pub const NUMS: [[u8; 8]; 10] = [N_0, N_1, N_2, N_3, N_4, N_5, N_6, N_7, N_8, N_9];

pub const N_0: [u8; 8] = [
    0b011110, 0b110011, 0b110011, 0b110011, 0b110011, 0b110011, 0b110011, 0b011110,
];
pub const N_1: [u8; 8] = [
    0b001100, 0b011100, 0b001100, 0b001100, 0b001100, 0b001100, 0b001100, 0b011110,
];
pub const N_2: [u8; 8] = [
    0b011110, 0b110011, 0b000011, 0b000110, 0b001100, 0b011000, 0b110000, 0b111111,
];
pub const N_3: [u8; 8] = [
    0b011110, 0b110011, 0b000011, 0b001110, 0b000011, 0b000011, 0b110011, 0b011110,
];
pub const N_4: [u8; 8] = [
    0b000011, 0b000111, 0b001111, 0b011011, 0b110011, 0b111111, 0b000011, 0b000011,
];
pub const N_5: [u8; 8] = [
    0b111111, 0b110000, 0b110000, 0b111110, 0b000011, 0b000011, 0b110011, 0b011110,
];
pub const N_6: [u8; 8] = [
    0b011110, 0b110011, 0b110000, 0b111110, 0b110011, 0b110011, 0b110011, 0b011110,
];
pub const N_7: [u8; 8] = [
    0b111111, 0b000011, 0b000110, 0b000110, 0b001100, 0b001100, 0b011000, 0b011000,
];
pub const N_8: [u8; 8] = [
    0b011110, 0b110011, 0b110011, 0b011110, 0b110011, 0b110011, 0b110011, 0b011110,
];
pub const N_9: [u8; 8] = [
    0b011110, 0b110011, 0b110011, 0b110011, 0b011111, 0b000011, 0b110011, 0b011110,
];
pub const PERCENT: [u8; 8] = [
    0b000000, 0b000000, 0b110001, 0b110010, 0b000100, 0b001000, 0b010011, 0b100011,
];
pub const CELSIUS: [u8; 8] = [
    0b000000, 0b000000, 0b100111, 0b001100, 0b001100, 0b001100, 0b001100, 0b000111,
];
pub const HUMIDITY: [u8; 8] = [
    0b0000000, 0b0000000, 0b0000000, 0b1000000, 0b1000000, 0b1110101, 0b1010101, 0b1010111,
];
pub const SEMICOLON: [u8; 8] = [0b00, 0b11, 0b11, 0b00, 0b00, 0b11, 0b11, 0b00];
pub const DOT: [u8; 8] = [0b00, 0b00, 0b00, 0b00, 0b00, 0b00, 0b11, 0b11];

// slim
pub const SLIM_NUMS: [[u8; 8]; 10] = [
    SLIM_N_0, SLIM_N_1, SLIM_N_2, SLIM_N_3, SLIM_N_4, SLIM_N_5, SLIM_N_6, SLIM_N_7, SLIM_N_8, SLIM_N_9,
];

pub const SLIM_N_0: [u8; 8] = [
    0b001110, 0b010001, 0b010001, 0b010001, 0b010001, 0b010001, 0b010001, 0b001110,
];
pub const SLIM_N_1: [u8; 8] = [
    0b000100, 0b001100, 0b000100, 0b000100, 0b000100, 0b000100, 0b000100, 0b001110,
];
pub const SLIM_N_2: [u8; 8] = [
    0b001110, 0b010001, 0b000001, 0b000010, 0b000100, 0b001000, 0b010000, 0b011111,
];
pub const SLIM_N_3: [u8; 8] = [
    0b001110, 0b010001, 0b000001, 0b000110, 0b000001, 0b000001, 0b010001, 0b001110,
];
pub const SLIM_N_4: [u8; 8] = [
    0b000001, 0b000011, 0b000101, 0b001001, 0b010001, 0b011111, 0b000001, 0b000001,
];
pub const SLIM_N_5: [u8; 8] = [
    0b011111, 0b010000, 0b010000, 0b011110, 0b000001, 0b000001, 0b010001, 0b001110,
];
pub const SLIM_N_6: [u8; 8] = [
    0b001110, 0b010001, 0b010000, 0b011110, 0b010001, 0b010001, 0b010001, 0b001110,
];
pub const SLIM_N_7: [u8; 8] = [
    0b011111, 0b000001, 0b000010, 0b000010, 0b000100, 0b000100, 0b000100, 0b000100,
];
pub const SLIM_N_8: [u8; 8] = [
    0b001110, 0b010001, 0b010001, 0b001110, 0b010001, 0b010001, 0b010001, 0b001110,
];
pub const SLIM_N_9: [u8; 8] = [
    0b001110, 0b010001, 0b010001, 0b010001, 0b001111, 0b000001, 0b010001, 0b001110,
];
pub const SLIM_PERCENT: [u8; 8] = [
    0b000000, 0b000000, 0b110001, 0b110010, 0b000100, 0b001000, 0b010011, 0b100011,
];
pub const SLIM_CELSIUS: [u8; 8] = [
    0b000000, 0b000000, 0b100111, 0b001000, 0b001000, 0b001000, 0b001000, 0b000111,
];
pub const SLIM_HUMIDITY: [u8; 8] = [
    0b0000000, 0b0000000, 0b0000000, 0b1000000, 0b1000000, 0b1110101, 0b1010101, 0b1010111,
];
pub const SLIM_SEMICOLON: [u8; 8] = [0b00, 0b00, 0b01, 0b00, 0b00, 0b01, 0b00, 0b00];
pub const SLIM_DOT: [u8; 8] = [0b00, 0b00, 0b00, 0b00, 0b00, 0b00, 0b00, 0b01];
