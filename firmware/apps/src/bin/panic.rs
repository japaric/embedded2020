#![no_main]
#![no_std]

use hal as _;
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    panic!()
}
