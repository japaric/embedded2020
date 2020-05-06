use core::sync::atomic::{AtomicBool, Ordering};

use pac::CLOCK;

#[tasks::declare]
mod task {
    use core::sync::atomic::Ordering;

    use pac::CLOCK;

    use crate::Interrupt0;

    use super::{Event, STARTED};

    fn init() {
        CLOCK::borrow_unchecked(|clock| {
            clock.TASKS_HFCLKSTART.write(|w| w.TASKS_HFCLKSTART(1));
            semidap::info!("started HFXO");

            unsafe { clock.INTENSET.write(|w| w.HFCLKSTARTED(1)) }
        });

        unsafe {
            crate::unmask0(&[Interrupt0::POWER_CLOCK]);
        }
    }

    fn CLOCK() -> Option<()> {
        semidap::trace!("CLOCK");

        let _ = Event::next()?;

        semidap::info!("HFXO is stable");
        STARTED.store(true, Ordering::Relaxed);

        None
    }
}

static STARTED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "radio")]
pub async fn has_stabilized() {
    use core::task::Poll;

    crate::poll_fn(|| {
        if STARTED.load(Ordering::Relaxed) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    })
    .await
}

#[cfg(feature = "usb")]
pub fn is_stable() -> bool {
    STARTED.load(Ordering::Relaxed)
}

enum Event {
    HFCLKSTARTED,
}

impl Event {
    fn next() -> Option<Self> {
        CLOCK::borrow_unchecked(|clock| {
            if clock.EVENTS_HFCLKSTARTED.read().bits() != 0 {
                clock.EVENTS_HFCLKSTARTED.zero();
                return Some(Event::HFCLKSTARTED);
            }

            None
        })
    }
}
