#![no_main]
#![no_std]

use cm::SCB;
use hal as _; // memory layout
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    semidap::info!("A");

    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_, w| w.PENDSVSET(1)));

    semidap::info!("C");

    semidap::exit(0)
}

#[no_mangle]
fn PendSV() {
    semidap::info!("ZZZ");
}
