#![no_main]
#![no_std]

use async_core::{task, unsync::Mutex};
use hal as _; // memory layout
use panic_never as _; // this program contains zero core::panic* calls

#[no_mangle]
fn main() -> ! {
    let m = Mutex::new(0);

    let mut lock = m.try_lock().unwrap();

    let a = async {
        semidap::info!("A: before lock");
        let lock = m.lock().await;
        semidap::info!("A: after lock");

        semidap::info!("A: {}", *lock);

        semidap::info!("DONE");

        semidap::exit(0)
    };

    let b = async {
        semidap::info!("B: before write");
        *lock = 42;
        drop(lock);

        semidap::info!("B: after releasing the lock");

        loop {
            semidap::info!("B: yield");
            task::r#yield().await;
        }
    };

    executor::run!(a, b)
}
