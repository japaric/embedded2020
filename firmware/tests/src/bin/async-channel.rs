#![no_main]
#![no_std]

use async_core::{task, unsync::spsc::Channel};
use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    let mut c = Channel::new();
    let (mut s, mut r) = c.split();

    let a = async {
        semidap::info!("A: before recv");
        let m = r.recv().await;
        semidap::info!("A: received `{}`", m);

        semidap::info!("DONE");
        semidap::exit(0)
    };

    let b = async {
        semidap::info!("B: before send");
        s.send(42).await;
        semidap::info!("B: after send");

        loop {
            semidap::info!("B: yield");
            task::r#yield().await;
        }
    };

    executor::run!(a, b)
}
