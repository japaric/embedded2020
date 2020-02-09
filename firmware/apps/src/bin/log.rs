#![no_main]
#![no_std]

use core::time::Duration;

use hal::time;
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    semidap::info!("Start");

    semidap::debug!("working..");
    // pretend we are doing some work
    while time::uptime() < Duration::from_millis(1) {
        continue;
    }

    semidap::error!("Something went wrong. Exiting..");

    semidap::exit(1);
}
