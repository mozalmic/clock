//use clock_macro::SnafuDebug;
use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub))]
pub enum Error<I2CError>
where
    I2CError: 'static + std::error::Error,
{
    #[snafu(display("AHT10 init error"))]
    InitError { source: I2CError },

    #[snafu(display("AHT10 measure error"))]
    MeasureError { source: I2CError },

    #[snafu(display("AHT10 is not calibrated yet"))]
    UncalibratedError {},
}
