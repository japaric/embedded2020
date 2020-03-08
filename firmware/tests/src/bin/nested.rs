//! (test) Stack backtrace across nested exceptions

#![no_main]
#![no_std]

use cm::SCB;
use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    // pend `PendSV`
    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_r, w| w.PENDSVSET(1)));

    use_the_stack();

    semidap::exit(0)
}

#[allow(non_snake_case)]
#[no_mangle]
fn PendSV() {
    use_the_stack();

    foo();
}

#[inline(never)]
fn foo() {
    // pend `NMI`
    SCB::borrow_unchecked(|scb| scb.ICSR.rmw(|_r, w| w.NMIPENDSET(1)));

    use_the_stack();
}

#[allow(non_snake_case)]
#[no_mangle]
fn NMI() {
    semidap::abort()
}

#[inline(always)]
fn use_the_stack() {
    let mut x = 0;
    let y = &mut x as *mut i32;
    unsafe { drop((&y as *const *mut i32).read_volatile()) }
}
