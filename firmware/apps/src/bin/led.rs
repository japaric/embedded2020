#![no_main]
#![no_std]

use hal::led;
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    led::Blue.on();

    semidap::exit(0)
}
