#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    // This operation does NOT halt the device
    semidap::info!("Hello, world!");

    // This halts the device and terminates the `semidap` instance running
    // on the host
    semidap::exit(0);
}
