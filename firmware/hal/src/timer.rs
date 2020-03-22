//! Timers

use core::{
    cmp,
    future::Future,
    pin::Pin,
    sync::atomic::{self, AtomicU8, Ordering},
    task::{Context, Poll, Waker},
    time::Duration,
};

use pac::RTC0;

use crate::{led, time, Interrupt0, NotSync};

/// [Singleton] timer
pub struct Timer {
    i: u8,
    // effectively owns the `RTC.CC*` register, which is `!Sync`
    _not_sync: NotSync,
}

static TAKEN: AtomicU8 = AtomicU8::new(0);

impl Timer {
    /// Claims the `Timer`
    // TODO allow claiming up to 4 instances
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
        semidap::panic!("`Timer` has already been claimed")
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match self.state {
            State::NotStarted { diff } => {
                let i = self.timer.i;
                let end = time::now().wrapping_add(diff);
                RTC0::borrow_unchecked(|rtc| {
                    if i == 0 {
                        rtc.CC0.write(|w| w.COMPARE(end))
                    } else if i == 1 {
                        rtc.CC1.write(|w| w.COMPARE(end))
                    } else if i == 2 {
                        rtc.CC2.write(|w| w.COMPARE(end))
                    } else {
                        rtc.CC3.write(|w| w.COMPARE(end))
                    }
                });
                self.state = State::Started { end };

                crate::mask0(&[Interrupt0::RTC0]);
                unsafe {
                    *WAKERS.get_unchecked_mut(usize::from(i)) = Some(cx.waker().clone());
                    // NOTE(fence) force the previous operation to complete before the interrupt is
                    // unmasked
                    atomic::compiler_fence(Ordering::Release);
                    crate::unmask0(&[Interrupt0::RTC0]); // volatile write
                }

                Poll::Pending
            }

            State::Started { end } => {
                // NOTE(defensive check) the user or some abstraction may poll this future before
                // the timer has expired
                if time::now() >= end {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

static mut WAKERS: [Option<Waker>; 4] = [None, None, None, None];

#[allow(non_snake_case)]
#[no_mangle]
fn RTC0() {
    RTC0::borrow_unchecked(|rtc| {
        if rtc.EVENTS_OVRFLW.read().EVENTS_OVRFLW() != 0 {
            led::Blue.off();
            led::Green.off();
            led::Red.on();
            semidap::abort();
        }

        if rtc.EVENTS_COMPARE0.read().EVENTS_COMPARE() != 0 {
            rtc.EVENTS_COMPARE0.zero();

            // NOTE(unsafe) uninterruptible operation because this runs at higher priority
            if let Some(waker) = unsafe { WAKERS[0].as_ref() } {
                waker.wake_by_ref();
            }
        }

        if rtc.EVENTS_COMPARE1.read().EVENTS_COMPARE() != 0 {
            rtc.EVENTS_COMPARE1.zero();

            if let Some(waker) = unsafe { WAKERS[1].as_ref() } {
                waker.wake_by_ref();
            }
        }

        if rtc.EVENTS_COMPARE2.read().EVENTS_COMPARE() != 0 {
            rtc.EVENTS_COMPARE2.zero();

            if let Some(waker) = unsafe { WAKERS[2].as_ref() } {
                waker.wake_by_ref();
            }
        }

        if rtc.EVENTS_COMPARE3.read().EVENTS_COMPARE() != 0 {
            rtc.EVENTS_COMPARE3.zero();

            if let Some(waker) = unsafe { WAKERS[3].as_ref() } {
                waker.wake_by_ref();
            }
        }
    });
}
