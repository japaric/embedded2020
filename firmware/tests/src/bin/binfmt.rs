//! Binary formatting registers

#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    let a = hal::cyccnt();

    cm::SCB::borrow_unchecked(|scb| {
        semidap::info!("{}", scb.AIRCR.read());
    });

    let b = hal::cyccnt();

    semidap::info!("That took {} cycles", b - a);

    semidap::exit(0)
}
