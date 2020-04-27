#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    foo(true);

    semidap::exit(0)
}

fn foo(recurse: bool) {
    let mut x = [0];
    let y = x.as_mut_ptr(); // use the stack
    unsafe {
        (&y as *const *mut i32).read_volatile();
    }

    if recurse {
        foo(false)
    } else {
        bar()
    }
}

fn bar() {
    semidap::abort()
}
