//! Zero-copy, async IEEE 802.15.4 radio loopback

#![no_main]
#![no_std]

use hal::radio;
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let (mut tx, mut rx) = radio::claim();

    let task = async {
        loop {
            let packet = rx.read().await;
            tx.write(packet).await;
        }
    };

    executor::run!(task)
}
