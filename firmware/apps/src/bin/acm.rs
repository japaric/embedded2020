#![deny(unused_must_use)]
#![no_main]
#![no_std]

use core::{fmt::Write as _, time::Duration};

use hal::{timer::Timer, usbd};
use heapless::{consts, String};
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let mut tx = usbd::serial();
    let mut timer = Timer::claim();
    let mut buf = String::<consts::U16>::new();

    let task = async {
        let mut i = 0;
        loop {
            buf.clear();
            writeln!(&mut buf, "Hello {}", i).ok();
            i += 1;
            tx.write(buf.as_bytes());
            timer.wait(Duration::from_secs(10)).await;
        }
    };

    executor::run!(task)
}
