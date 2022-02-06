use chrono::Timelike;

use crate::{
    display::{self, LinearMatrixDisplay},
    error::Error,
};

pub(crate) fn draw(display: &mut LinearMatrixDisplay, draw_dots: bool, slim: bool) -> Result<(), Error> {
    let time = chrono::Local::now();
    let hours = time.hour();
    let minutes = time.minute();

    let (semicolon, nums) = if slim {
        (display::SLIM_SEMICOLON, display::SLIM_NUMS)
    } else {
        (display::SEMICOLON, display::NUMS)
    };

    let h1 = nums[(hours / 10) as usize];
    let h2 = nums[(hours % 10) as usize];
    let m1 = nums[(minutes / 10) as usize];
    let m2 = nums[(minutes % 10) as usize];

    display.draw(|x, y| {
        if x >= 1 && x <= 6 {
            h1[y] & (1 << (6 - x))
        } else if x >= 8 && x <= 13 {
            h2[y] & (1 << (13 - x))
        } else if x >= 15 && x <= 16 && draw_dots {
            semicolon[y] & (1 << (16 - x))
        } else if x >= 18 && x <= 23 {
            m1[y] & (1 << (23 - x))
        } else if x >= 25 && x <= 30 {
            m2[y] & (1 << (30 - x))
        } else {
            0
        }
    })?;

    Ok(())
}
