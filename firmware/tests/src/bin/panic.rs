#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    semidap::panic!("bye")
}
