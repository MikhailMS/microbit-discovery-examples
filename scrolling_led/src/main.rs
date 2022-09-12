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
    hal::{
        clocks::Clocks,
        rtc::{Rtc, RtcInterrupt},
    },
    pac::{ self, interrupt, RTC0, TIMER1 },
};

use microbit_text_scroller::{Animate, ScrollMessage};


// TIMER1 drives the display
// RTC0   drives the animation
static DISPLAY:     Mutex<RefCell<Option<Display<TIMER1>>>>   = Mutex::new(RefCell::new(None));
static ENTRY_TIMER: Mutex<RefCell<Option<Rtc<RTC0>>>>         = Mutex::new(RefCell::new(None));
// SCROLL_ALPH contains the text that we would be scrolling right to left
static SCROLL_ALPH: Mutex<RefCell<Option<ScrollMessage<104>>>> = Mutex::new(RefCell::new(None));


#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Board is not available");

    // Starting the low-frequency clock (needed for RTC to work)
    Clocks::new(board.CLOCK).start_lfclk();

    // RTC at 16Hz (32_768 / (2047 + 1)) -->
    // refresh interval is 62.5ms
    // !! However cannot be greater than 4095 !!
    let mut rtc0 = Rtc::new(board.RTC0, 4095).expect("RTC is not available");
    rtc0.enable_event(RtcInterrupt::Tick);
    rtc0.enable_interrupt(RtcInterrupt::Tick, None);
    rtc0.enable_counter();

    let display  = Display::new(board.TIMER1, board.display_pins);
    let scroller = ScrollMessage::new("SCROLLINGLED");

    cortex_m::interrupt::free(move |cs| {
        DISPLAY.borrow(cs).replace(Some(display));
        ENTRY_TIMER.borrow(cs).replace(Some(rtc0));
        SCROLL_ALPH.borrow(cs).replace(Some(scroller));
    });

    unsafe {
        // Would work without these 2 lines, but seems like setting priority is important when doing
        // interrupts
        board.NVIC.set_priority(pac::Interrupt::RTC0, 64);
        board.NVIC.set_priority(pac::Interrupt::TIMER1, 128);

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

        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            if let Some(scroller) = SCROLL_ALPH.borrow(cs).borrow_mut().as_mut() {
                if !scroller.is_finished() {
                    FRAME.set(scroller.next());
                    display.show_frame(FRAME);
                } else {
                    display.clear();
                    scroller.reset();
                }
            }
        }
    });
}
