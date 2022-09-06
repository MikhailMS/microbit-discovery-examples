#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use lsm303agr::{
    AccelOutputDataRate,
    AccelScale,
    Lsm303agr,
};
use microbit::{
    hal::Timer,
    hal::prelude::*,
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A
};
use nb::Error::WouldBlock;


#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = microbit::Board::take().unwrap();
    let i2c   = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut countdown = Timer::new(board.TIMER0);
    let mut delay     = Timer::new(board.TIMER1);
    let mut sensor    = Lsm303agr::new_with_i2c(i2c);

    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    sensor.set_accel_scale(AccelScale::G8).unwrap();

    let mut max_punch: i32 = 0;
    let mut enable_measure = false;

    loop {
        while !sensor.accel_status().unwrap().xyz_new_data {}
        let data = sensor.accel_data().unwrap();
        
        if enable_measure {
            match countdown.wait() {
                Err(WouldBlock) => {
                    if data.x > max_punch {
                        max_punch = data.x
                    }
                },
                Ok(_)           => {
                    enable_measure = false;
                    rprintln!("Max acceleration: (x) = ({})", max_punch);
                    
                },
                Err(_)          => unreachable!()
                
            }
        } else {
            if data.x > 300 {
                enable_measure = true;
                max_punch      = data.x;

                countdown.start(10000_u32);
            }
        }

        delay.delay_ms(50_u8);
    }
}
