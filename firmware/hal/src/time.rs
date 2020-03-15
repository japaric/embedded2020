//! Temporal quantification

use core::{ops, time::Duration};

use pac::RTC0;

/// A measurement of a monotonically nondecreasing clock. Opaque and only useful
/// with `core::time::Duration`
pub struct Instant {
    // TODO turn into `u64`
    inner: u32,
}

impl Instant {
    /// Returns an `Instant` corresponding to "now"
    pub fn now() -> Self {
        Instant {
            inner: RTC0::borrow_unchecked(|rtc| rtc.COUNTER.read().into()),
        }
    }
}

impl ops::Sub for Instant {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Duration {
        semidap::assert!(
            self.inner >= rhs.inner,
            "supplied instant is later than self"
        );

        // `ticks` is always less than `1 << 24`
        let ticks = self.inner.wrapping_sub(rhs.inner);
        let secs = ticks >> 15;
        // one tick is equal to `1e9 / 32768` nanos
        // the fraction can be reduced to `1953125 / 64`
        // which can be further decomposed as `78125 * (5 / 4) * (5 / 4) * (1 /
        // 4)`. Doing the operation this way we can stick to 32-bit arithmetic
        // without overflowing the value at any stage
        let nanos =
            (((ticks % 32768).wrapping_mul(78125) >> 2).wrapping_mul(5) >> 2).wrapping_mul(5) >> 2;
        Duration::new(secs.into(), nanos)
    }
}

/// Returns the time elapsed since the last reset (either POR or soft reset)
pub fn uptime() -> Duration {
    Instant::now() - Instant { inner: 0 }
}
