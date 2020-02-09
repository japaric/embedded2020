#![no_main]
#![no_std]

use hal as _;
use panic_abort as _; // panic handler // memory layout

#[no_mangle]
fn main() -> ! {
    // this tries to read non-existent memory and causes a
    // `HardFault` (hardware) exception
    unsafe {
        (0xffff_fff0 as *const u32).read_volatile();
    }

    semidap::exit(0);
}
