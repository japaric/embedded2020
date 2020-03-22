#![no_main]
#![no_std]

use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls
use async_core::task;

#[no_mangle]
fn main() -> ! {
    let a = async {
        semidap::info!("before yield");
        task::r#yield().await;
        semidap::info!("after yield");
        semidap::exit(0)
    };

    executor::run!(a)
}
