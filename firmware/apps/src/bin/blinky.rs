#![no_main]
#![no_std]

use core::time::Duration;

use hal::{led, timer::Timer};
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    let dur = Duration::from_secs(1);
    let mut timer = Timer::claim();

    let blinky = async {
        loop {
            led::Blue.on();
            timer.wait(dur).await;

            led::Blue.off();
            timer.wait(dur).await;
        }
    };

    executor::run!(blinky)
}
