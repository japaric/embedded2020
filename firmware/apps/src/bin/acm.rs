#![deny(unused_must_use)]
#![no_main]
#![no_std]

use hal::usbd;
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let mut tx = usbd::serial();

    let task = async {
        tx.write(b"Hello\nworld\n");

        loop {}
    };

    executor::run!(task)
}
