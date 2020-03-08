//! (test) Stack backtrace across nested exceptions

#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    semidap::info!("I am {}", hal::deviceid().to_le_bytes()[..]);

    semidap::exit(0)
}
