#![no_main]
#![no_std]

use cm::SCB;
use hal as _; // memory layout
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    semidap::info!("A");

    // trigger `PendSV`
    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_, w| w.PENDSVSET(1)));

    semidap::info!("B");

    semidap::exit(0)
}

#[allow(non_snake_case)]
#[no_mangle]
fn PendSV() {
    semidap::info!("ZZZ");
}
