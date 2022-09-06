#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::fmt::Write;
use core::str::from_utf8;
use cortex_m_rt::entry;
use heapless::Vec;
use lsm303agr::{ AccelOutputDataRate, Lsm303agr, MagOutputDataRate };
use microbit::{
    hal::twim,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
    pac::twim0::frequency::FREQUENCY_A,
};
use panic_rtt_target as _;
use rtt_target::{ rtt_init_print, rprintln };


const MAG_COMMAND: &str = "magnetometer";
const ACC_COMMAND: &str = "accelerometer";


#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Init board & I2C communication
    let board = microbit::Board::take().unwrap();
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut serial = uarte::Uarte::new(
        board.UARTE0,
        board.uart.into(),
        Parity::EXCLUDED,
        Baudrate::BAUD115200,
    );

    // Init LSM303ARG sensor
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();

    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();

    let mut buffer: Vec<u8, 14> = Vec::new();
    let uarte_rx_buf            = &mut [0u8; 1][..];

    loop {
        buffer.clear();
        
        loop {
            serial.read(uarte_rx_buf).unwrap();
            let byte = uarte_rx_buf[0];
            
            if buffer.push(byte).is_err() {
                write!(serial, "error: buffer full\r\n").unwrap();
                break;
            }

            if byte == 13 {
                let command = from_utf8(&buffer).unwrap().trim();

                if command == MAG_COMMAND {
                    let data = nb::block!(sensor.mag_data()).unwrap();

                    rprintln!("Magnetic field: (x, y, z) = ({}, {}, {})", data.x, data.y, data.z);
                    write!(serial, "Magnetic field: (x, y, z) = ({}, {}, {})\r\n", data.x, data.y, data.z).unwrap();
                } else if command == ACC_COMMAND {
                    while !sensor.accel_status().unwrap().xyz_new_data {}
                    let data = sensor.accel_data().unwrap();

                    rprintln!("Acceleration: (x, y, z) = ({}, {}, {})", data.x, data.y, data.z);
                    write!(serial, "Acceleration: (x, y, z) = ({}, {}, {})\r\n", data.x, data.y, data.z).unwrap();
                } else {
                    rprintln!("error: [{}] wrong command\r\n", command);
                    write!(serial, "error: [{}] wrong command\r\n", command).unwrap();
                }
                break;
            }
        }
    }
}


// #![deny(unsafe_code)]
// #![no_main]
// #![no_std]

// use cortex_m_rt::entry;
// use rtt_target::{rtt_init_print, rprintln};
// use panic_rtt_target as _;

// use microbit::hal::prelude::*;
// use microbit::{
//     hal::twim,
//     pac::twim0::frequency::FREQUENCY_A,
// };

// const ACCELEROMETER_ADDR: u8 = 0b0011001;
// const MAGNETOMETER_ADDR: u8  = 0b0011110;

// const ACCELEROMETER_ID_REG: u8 = 0x0f;
// const MAGNETOMETER_ID_REG: u8  = 0x4f;

// const ACCELEROMETER_REGISTRY: u8 = 0b110011;
// const MAGNETOMETER_REGISTRY: u8  = 0b1000000;


// #[entry]
// fn main() -> ! {
//     rtt_init_print!();

//     let board   = microbit::Board::take().unwrap();
//     let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

//     let mut acc = [0];
//     let mut mag = [0];

//     // First write the address + register onto the bus, then read the chip's responses
//     i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc).unwrap();
//     i2c.write_read(MAGNETOMETER_ADDR,  &[MAGNETOMETER_ID_REG],  &mut mag).unwrap();

//     rprintln!("The accelerometer chip's id is: {:#b}", acc[0]);
//     rprintln!("The magnetometer chip's id is: {:#b}",  mag[0]);

//     loop {}
// }
