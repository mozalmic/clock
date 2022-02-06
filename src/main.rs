//mod aht10;
mod aht10;
mod clock;
mod display;
mod error;
mod model;
mod weather;

use aht10::AHT10;
use clap::{load_yaml, App, AppSettings};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use linux_embedded_hal::{Delay, I2cdev};
use model::Config;
use snafu::ResultExt;
use std::path::Path;
use sysfs_pwm::Pwm;

use crate::{display::LinearMatrixDisplay, weather::WeatherType};

#[macro_use]
extern crate bitflags;

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("cli.yaml");
    let opts = App::from_yaml(yaml)
        .setting(AppSettings::AllowLeadingHyphen)
        .setting(AppSettings::SubcommandsNegateReqs)
        .get_matches();

    // get subcommand
    match opts.subcommand() {
        ("init-config", Some(opts)) => init_config(opts)?,

        ("test-pwm", Some(opts)) => test_pwm(opts)?,

        ("test-lirc", Some(opts)) => test_lirc(opts)?,

        _ => {
            let config_location = opts.value_of("config").unwrap_or("./clock.yaml");
            let do_clean = opts.is_present("clean");
            run(&Path::new(&config_location), do_clean).await?;
        }
    }

    println!("FINISHED.");

    Ok(())
}

fn init_config(opts: &clap::ArgMatches) -> Result<(), error::Error> {
    let config_location = opts.value_of("config").unwrap_or("./clock.yaml");
    let config = Config::new();
    config.to_yaml(Path::new(config_location))?;
    Ok(())
}

enum PwmAction {
    ENABLE { pwm: u32, frequiency: u32, duty: u32 },
    DISABLE { pwm: u32 },
    NONE,
}

fn test_pwm(opts: &clap::ArgMatches) -> Result<(), error::Error> {
    let pwm = opts.value_of("INPUT").unwrap().parse().unwrap_or(0);
    let action = if opts.is_present("disable") {
        PwmAction::DISABLE { pwm: pwm }
    } else if let Some(f) = opts.value_of("enable") {
        PwmAction::ENABLE {
            pwm: pwm,
            frequiency: f.parse().unwrap_or(0),
            duty: opts.value_of("duty").unwrap().parse().unwrap_or(100),
        }
    } else {
        PwmAction::NONE
    };

    match action {
        PwmAction::ENABLE { pwm, frequiency, duty } => {
            let pwm = Pwm::new(0, pwm).unwrap(); // number depends on chip, etc.
            pwm.enable(true).unwrap();
            let period = 1_000_000_000 / frequiency; // nanoseconds
            let duty_time = period * duty / 100;
            pwm.set_period_ns(1_000_000_000 / frequiency).unwrap();
            pwm.set_duty_cycle_ns(duty_time).unwrap();
        }
        PwmAction::DISABLE { pwm } => {
            let pwm = Pwm::new(0, pwm).unwrap(); // number depends on chip, etc.
            pwm.enable(false).unwrap();
        }
        PwmAction::NONE => {
            println!("No action applied to PWM");
        }
    }

    Ok(())
}

fn test_lirc(opts: &clap::ArgMatches) -> Result<(), error::Error> {
    /*if let Ok(f) = File::open("/dev/lirc0") {
        let r = f.read(buf)
    }*/
    Ok(())
}

async fn run(config_location: &Path, do_clean: bool) -> Result<(), error::Error> {
    // read config
    let config = Config::from_yaml(config_location)?;

    // initialize screen
    let mut display = LinearMatrixDisplay::new(
        &config.display.gpio_dev,
        config.display.data_pin,
        config.display.cs_pin,
        config.display.clk_pin,
        config.display.number_of_matrices,
        config.display.brightness,
    )?;
    if do_clean {
        return Ok(());
    }
    println!(
        "Clock started on `{}` device, ports: [data={}, cs={}, clk={}]",
        &config.display.gpio_dev, config.display.data_pin, config.display.cs_pin, config.display.clk_pin
    );

    let mut d = Delay;
    // initialize humidity and temperature sensor
    let mut sensor = AHT10 {
        i2c: I2cdev::new(&config.weather.sensor.gpio_dev).context(error::I2CSnafu)?,
        delay_ms: |ms| Delay {}.delay_ms(ms),
    };
    sensor.init().context(error::SensorSnafu)?;

    // draw in cycle
    let mut weather_interwal_counter = 0;
    loop {
        if weather_interwal_counter >= config.weather.display_interval_sec {
            weather_interwal_counter = 0;

            weather::draw(&mut display, &mut sensor, WeatherType::Temperature, config.display.slim)?;
            d.delay_ms(config.weather.temperature_on_display_msec);

            weather::draw(&mut display, &mut sensor, WeatherType::Humidity, config.display.slim)?;
            d.delay_ms(config.weather.humidity_on_display_msec);
        } else {
            weather_interwal_counter += 1;

            clock::draw(&mut display, true, config.display.slim)?;
            d.delay_ms(500u32);

            clock::draw(&mut display, false, config.display.slim)?;
            d.delay_ms(500u32);
        }
    }
}
