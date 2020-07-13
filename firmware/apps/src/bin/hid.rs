#![deny(unused_must_use)]
#![no_main]
#![no_std]

use hal::usbd::{self, Packet};
use panic_abort as _;

#[no_mangle]
fn main() -> ! {
    let (mut hidout, mut hidin) = usbd::hid();

    let task = async {
        let mut packet = Packet::new().await;
        hidout.recv(&mut packet).await;
        hidin.send(&packet).await;
        hidin.flush().await;
        semidap::exit(0)
    };

    executor::run!(task)
}
