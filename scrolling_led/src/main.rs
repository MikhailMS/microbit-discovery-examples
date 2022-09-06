#![no_main]
#![no_std]


use panic_halt as _;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use microbit::{
    board::Board,
    display::nonblocking::{
        Display,
        Frame,
        MicrobitFrame,
    },
    // display::blocking::Display as BDisplay,
    hal::{
        clocks::Clocks,
        rtc::{Rtc, RtcInterrupt},
        Timer,
    },
    pac::{self, interrupt, RTC0, TIMER0, TIMER1},
};

use microbit_text_scroller::{Animate, ScrollMessage};


// TIMER1 drives the display
// RTC0   drives the animation
// static MAIN_TIMER:  Mutex<RefCell<Option<Timer<TIMER0>>>>     = Mutex::new(RefCell::new(None));
static DISPLAY:     Mutex<RefCell<Option<Display<TIMER1>>>>   = Mutex::new(RefCell::new(None));
static ENTRY_TIMER: Mutex<RefCell<Option<Rtc<RTC0>>>>         = Mutex::new(RefCell::new(None));
// SCROLL_ALPH contains the text that we would be scrolling right to left
static SCROLL_ALPH: Mutex<RefCell<Option<ScrollMessage<64>>>> = Mutex::new(RefCell::new(None));

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
    let mut board = Board::take().unwrap();

    // Starting the low-frequency clock (needed for RTC to work)
    Clocks::new(board.CLOCK).start_lfclk();

    // RTC at 16Hz (32_768 / (2047 + 1)) -->
    // refresh interval is 62.5ms
    // !! However cannot be greater than 4095 !!
    let mut rtc0 = Rtc::new(board.RTC0, 4095).unwrap();
    rtc0.enable_event(RtcInterrupt::Tick);
    rtc0.enable_interrupt(RtcInterrupt::Tick, None);
    rtc0.enable_counter();

    // let mut timer = Timer::new(board.TIMER0);

    let display  = Display::new(board.TIMER1, board.display_pins);
    let scroller = ScrollMessage::new("ABCDEFGH");

    cortex_m::interrupt::free(move |cs| {
        *DISPLAY.borrow(cs).borrow_mut()     = Some(display);
        *ENTRY_TIMER.borrow(cs).borrow_mut() = Some(rtc0);
        // *MAIN_TIMER.borrow(cs).borrow_mut()  = Some(timer);
        *SCROLL_ALPH.borrow(cs).borrow_mut() = Some(scroller);
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::TIMER1);
        pac::NVIC::unmask(pac::Interrupt::RTC0);
    }

    loop {}
}

#[interrupt]
fn TIMER1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}

#[interrupt]
fn RTC0() {
    static mut FRAME: MicrobitFrame = MicrobitFrame::default();

    cortex_m::interrupt::free(|cs| {
        if let Some(rtc) = ENTRY_TIMER.borrow(cs).borrow_mut().as_mut() {
            rtc.reset_event(RtcInterrupt::Tick);
        }
    });

    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            if let Some(scroller) = SCROLL_ALPH.borrow(cs).borrow_mut().as_mut() {
                if !scroller.is_finished() {
                    FRAME.set(scroller.next());
                    display.show_frame(FRAME);
                } else {
                    display.clear();
                    scroller.reset();
                    // unsafe {
                    //     // pac::NVIC::mask(pac::Interrupt::TIMER1);
                    //     pac::NVIC::mask(pac::Interrupt::RTC0);
                    //     pac::NVIC::unmask(pac::Interrupt::TIMER0);
                    // }
                }
            }
        }
    });
}

// #[interrupt]
// fn TIMER0() {
//     static mut last_led: (i32, i32) = (0,0);
//     static mut leds: [[i32; 5]; 5]  = [
//         [0, 0, 0, 0, 0],
//         [0, 0, 0, 0, 0],
//         [0, 0, 0, 0, 0],
//         [0, 0, 0, 0, 0],
//         [0, 0, 0, 0, 0],
//     ];

//     cortex_m::interrupt::free(|cs| {
//         if let Some(timer) = ENTRY_TIMER.borrow(cs).borrow_mut().as_mut() {
//             timer.reset_event();
//         }
//     });

//     cortex_m::interrupt::free(|cs| {
//         if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
//             if let Some(timer) = MAIN_TIMER.borrow(cs).borrow_mut().as_mut() {
//                 for current_led in NEW_ROULETTE_PATH.iter() {
//                     leds[last_led.0][last_led.1]       = 0;
//                     leds[current_led.0][current_led.1] = 1;

//                     display.show(leds);
//                     timer.delay_ms(DELAY_MS);
//                     // display.clear()
//                     // timer.delay_ms(250_u32);
//                     last_led = *current_led;
//                 }

//                 unsafe {
//                     pac::NVIC::mask(pac::Interrupt::TIMER0);
//                     pac::NVIC::unmask(pac::Interrupt::RTC0);
//                 }
//             }

//         }
//     });
// }
