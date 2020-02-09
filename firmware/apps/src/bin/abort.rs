#![no_main]
#![no_std]

use hal as _;
use panic_abort as _; // panic handler

#[no_mangle]
fn main() -> ! {
    foo(true);

    semidap::exit(0)
}

fn foo(recurse: bool) {
    let mut x = [0];
    let y = x.as_mut_ptr(); // use the stack
    unsafe { drop((&y as *const *mut i32).read_volatile()) }

    if recurse {
        foo(false)
    } else {
        bar()
    }
}

fn bar() {
    semidap::abort()
}
