//! Timers

use core::{
    cmp,
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU8, Ordering},
    task::{Context, Poll},
    time::Duration,
};

use pac::RTC0;

use crate::{time, NotSync};

const STEP: u32 = 4096; // 125 ms

#[tasks::declare]
mod task {
    use core::sync::atomic::{AtomicU16, Ordering};

    use pac::RTC0;

    use crate::{led, Interrupt0};

    use super::STEP;

    fn init() {
        RTC0::borrow_unchecked(|rtc| unsafe {
            rtc.INTENSET.write(|w| w.COMPARE0(1).OVRFLW(1));
            rtc.CC0.write(|w| w.COMPARE(super::STEP));
        });

        led::Red.on();

        unsafe { crate::unmask0(&[Interrupt0::RTC0]) }
    }

    fn RTC0() {
        RTC0::borrow_unchecked(|rtc| {
            if rtc.EVENTS_OVRFLW.read().EVENTS_OVRFLW() != 0 {
                semidap::error!("RTC count overflowed ... aborting");
                led::Blue.off();
                led::Green.off();
                led::Red.on();
                semidap::abort();
            }

            if rtc.EVENTS_COMPARE0.read().EVENTS_COMPARE() != 0 {
                static COUNT: AtomicU16 = AtomicU16::new(0);

                rtc.EVENTS_COMPARE0.zero();
                let count = COUNT.load(Ordering::Relaxed).wrapping_add(1);
                rtc.CC0
                    .write(|w| w.COMPARE(u32::from(count.wrapping_add(1)) * STEP));
                COUNT.store(count, Ordering::Relaxed);

                match count % 12 {
                    0 => led::Red.on(),
                    1 => led::Red.off(),
                    2 => led::Red.on(),
                    3 => led::Red.off(),
                    _ => {}
                }
            }

            if rtc.EVENTS_COMPARE1.read().EVENTS_COMPARE() != 0 {
                rtc.EVENTS_COMPARE1.zero();
                rtc.INTENCLR.write(|w| w.COMPARE1(1));
            }

            if rtc.EVENTS_COMPARE2.read().EVENTS_COMPARE() != 0 {
                rtc.EVENTS_COMPARE2.zero();
                rtc.INTENCLR.write(|w| w.COMPARE2(1));
            }

            if rtc.EVENTS_COMPARE3.read().EVENTS_COMPARE() != 0 {
                rtc.EVENTS_COMPARE3.zero();
                rtc.INTENCLR.write(|w| w.COMPARE3(1));
            }
        });
    }
}

/// [Singleton] timer
pub struct Timer {
    i: u8,
    // effectively owns the `RTC.CC*` register, which is `!Sync`
    _not_sync: NotSync,
}

// NOTE timer `0` is used for the "heartbeat" task
static TAKEN: AtomicU8 = AtomicU8::new(1);

impl Timer {
    /// Claims the `Timer`
    pub fn claim() -> Self {
        if TAKEN.load(Ordering::Relaxed) < 4 {
            let i = TAKEN.fetch_add(1, Ordering::Relaxed);

            if i < 4 {
                return Timer {
                    i,
                    _not_sync: NotSync::new(),
                };
            }
        }

        semidap::panic!("no more `Timer` instances can be claimed")
    }

    /// Waits for the specified duration
    pub fn wait<'t>(&'t mut self, dur: Duration) -> impl Future<Output = ()> + 't {
        let diff = dur.as_secs() as u32 * 32_768
            + dur
                .subsec_nanos()
                .wrapping_mul(4)
                .wrapping_div(5)
                .wrapping_mul(4)
                .wrapping_div(5)
                .wrapping_mul(4)
                .wrapping_div(78125);

        Wait {
            timer: self,
            state: State::NotStarted {
                diff: cmp::max(diff, 2),
            },
        }
    }
}

struct Wait<'a> {
    timer: &'a mut Timer,
    state: State,
}

#[derive(Clone, Copy)]
enum State {
    NotStarted { diff: u32 },
    Started { end: u32 },
}

impl Future for Wait<'_> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        match self.state {
            State::NotStarted { diff } => {
                let i = self.timer.i;
                let end = time::now().wrapping_add(diff);
                RTC0::borrow_unchecked(|rtc| {
                    // if i == 0 {
                    //     rtc.CC0.write(|w| w.COMPARE(end))
                    // } else
                    if i == 1 {
                        rtc.CC1.write(|w| w.COMPARE(end));
                        unsafe { rtc.INTENSET.write(|w| w.COMPARE1(1)) }
                    } else if i == 2 {
                        rtc.CC2.write(|w| w.COMPARE(end));
                        unsafe { rtc.INTENSET.write(|w| w.COMPARE2(1)) }
                    } else {
                        rtc.CC3.write(|w| w.COMPARE(end));
                        unsafe { rtc.INTENSET.write(|w| w.COMPARE3(1)) }
                    }
                });
                self.state = State::Started { end };

                Poll::Pending
            }

            State::Started { end } => {
                if time::now() >= end {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}
