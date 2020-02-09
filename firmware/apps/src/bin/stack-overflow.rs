#![no_main]
#![no_std]

use hal as _;
use panic_abort as _; // panic handler
use semidap::println;

#[no_mangle]
fn main() -> ! {
    fib(15);

    semidap::exit(0);
}

fn fib(n: u32) -> u32 {
    let mut x = [n; 8 * 1024]; // allocate a 32 KB buffer on the stack
    println!("SP = {:?}", x.as_mut_ptr());

    if n < 2 {
        1
    } else {
        fib(n - 1).wrapping_add(fib(n - 2))
    }
}
