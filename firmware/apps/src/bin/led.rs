#![no_main]
#![no_std]

use hal::led;
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    led::Blue.on();

    semidap::exit(0)
}
