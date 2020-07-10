//! Zero-copy, async IEEE 802.15.4 radio loopback

#![no_main]
#![no_std]

use hal::radio::{self, Channel};
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let (mut tx, _) = radio::claim(Channel::_20);

    let task = async {
        let mut packet = radio::Packet::new().await;
        packet.copy_from_slice(b"hello");
        tx.write(&packet).await.ok();
        semidap::exit(0);
    };

    executor::run!(task)
}
