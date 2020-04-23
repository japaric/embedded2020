//! async USB <-> IEEE 802.15.4 radio bridge

#![no_main]
#![no_std]

use hal::{radio, usbd};
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let (mut epin1, mut epout1) = usbd::claim(); // bulk endpoints
    let (mut tx, mut rx) = radio::claim();

    let t1 = async {
        loop {
            let packet = rx.read().await;

            if let Ok(packet) = usbd::Packet::try_from(packet) {
                epin1.write(packet).await; // radio packet fits in one USB packet
            } else {
                semidap::error!("unimplemented: discarding large radio packet");
            }
        }
    };

    let t2 = async {
        loop {
            let packet = epout1.read().await;
            tx.write(packet.into()).await; // no-op: one USB packet always fits in one radio packet
        }
    };

    executor::run!(t1, t2)
}
