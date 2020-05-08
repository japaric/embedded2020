#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

// the heartbeat task runs in the background so we just sleep here
#[no_mangle]
fn main() -> ! {
    loop {
        asm::wfi()
    }
}
