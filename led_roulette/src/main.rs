#![deny(unsafe_code)]
#![no_main]
#![no_std]


use cortex_m_rt::entry;
use panic_halt as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::Timer,
};

const NEW_ROULETTE_PATH: [(usize, usize); 48] = [
    (0,0), (0,1), (0,2), (0,3), (0,4),
    (1,4), (2,4), (3,4), (4,4),
    (4,3), (4,2), (4,1), (4,0),
    (3,0), (2,0), (1,0),
    (1,1), (1,2), (1,3),
    (2,3), (3,3),
    (3,2), (3,1),
    (2,1),
    (2,2),
    (2,1),
    (3,1), (3,2),
    (3,3), (2,3),
    (1,3), (1,2), (1,1),
    (1,0), (2,0), (3,0),
    (4,0), (4,1), (4,2), (4,3),
    (4,4), (3,4), (2,4), (1,4),
    (0,4), (0,3), (0,2), (0,1)
];
    
const DELAY_MS: u32 = 40;

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();

    let mut timer   = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut last_led = (0,0);
    let mut leds     = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    loop {
        for current_led in NEW_ROULETTE_PATH.iter() {
            leds[last_led.0][last_led.1]       = 0;
            leds[current_led.0][current_led.1] = 1;

            display.show(&mut timer, leds, DELAY_MS);
            last_led = *current_led;
        }
    }
}
