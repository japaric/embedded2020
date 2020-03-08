#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    // this tries to read non-existent memory and causes a
    // `HardFault` (hardware) exception
    unsafe {
        (0xffff_fff0 as *const u32).read_volatile();
    }

    semidap::exit(0);
}
