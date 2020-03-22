//! Tasks

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

struct Yield {
    yielded: bool,
}

impl Future for Yield {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            cx.waker().wake_by_ref(); // wake ourselves

            Poll::Pending
        }
    }
}

/// Suspends the current task
pub fn r#yield() -> impl Future<Output = ()> {
    Yield { yielded: false }
}
