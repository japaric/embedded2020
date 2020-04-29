//! Zero-copy, interrupt-driven, async USB bulk function

#![no_main]
#![no_std]

use hal::usbd;
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let (mut epin1, mut epout1) = usbd::claim(); // bulk endpoints

    let task = async {
        loop {
            let mut packet = epout1.read().await; // host -> device
            packet.reverse(); // reverse the host data in place
            epin1.write(packet).await; // device -> host
        }
    };

    executor::run!(task)
}
