#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;
use cortex_m_rt::entry;
use libm::atan2f;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{
    display::blocking::Display,
    hal::Timer,
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A
};
use lsm303agr::{
    AccelOutputDataRate,
    Lsm303agr,
    MagOutputDataRate
};

mod calibration;
mod led;
use calibration::{ calc_calibration, calibrated_measurement };
use led::{ Direction, direction_to_led };


#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = microbit::Board::take().unwrap();
    let i2c   = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer   = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();

    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();

    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);

    rprintln!("Calibration: {:?}", calibration);
    rprintln!("Calibration done, entering busy loop");

    loop {
        while !sensor.mag_status().unwrap().xyz_new_data {}

        let mut data = sensor.mag_data().unwrap();
        data         = calibrated_measurement(data, &calibration);

        let theta = atan2f(data.y as f32, data.x as f32);

        let dir = if theta < -9.*PI/10. {
            Direction::West
        } else if theta < -7.*PI/10. {
            Direction::SouthWest
        } else if theta < -5.*PI/10. {
            Direction::South
        } else if theta < -3.*PI/10. {
            Direction::SouthEast
        } else if theta < 3.*PI/10. {
            Direction::East
        } else if theta < 5.*PI/10. {
            Direction::NorthEast
        } else if theta < 7.*PI/10. {
            Direction::North
        } else if theta < 9.*PI/10. {
            Direction::NorthWest
        } else {
            Direction::West
        };

        display.show(&mut timer, direction_to_led(dir), 1000);
    }
}
