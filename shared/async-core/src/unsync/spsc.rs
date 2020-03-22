//! Single Producer Single Consumer channels

use core::{
    cell::{Cell, UnsafeCell},
    future::Future,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr,
    task::{Context, Poll, Waker},
};

/// `async`-aware channel
// TODO user configurable capacity
pub struct Channel<T> {
    buffer: UnsafeCell<MaybeUninit<T>>,
    full: Cell<bool>,
    recv_waker: Cell<Option<Waker>>,
    send_waker: Cell<Option<Waker>>,
}

impl<T> Channel<T> {
    /// Creates a new channel
    pub const fn new() -> Self {
        Self {
            buffer: UnsafeCell::new(MaybeUninit::uninit()),
            full: Cell::new(false),
            recv_waker: Cell::new(None),
            send_waker: Cell::new(None),
        }
    }

    /// Splits the channel in `sender` and `receiver` endpoints
    pub fn split<'c>(&'c mut self) -> (Sender<'c, T>, Receiver<'c, T>) {
        let channel = self;
        (Sender { channel }, Receiver { channel })
    }
}

/// Sending side of a channel
pub struct Sender<'c, T> {
    channel: &'c Channel<T>,
}

impl<T> Sender<'_, T> {
    /// Sends a message into the channel
    pub fn send<'s>(&'s mut self, msg: T) -> impl Future<Output = ()> + 's {
        struct Send<'s, 'c, T> {
            msg: ManuallyDrop<T>,
            sender: &'s Sender<'c, T>,
            sent: Cell<bool>,
        }

        impl<T> Future for Send<'_, '_, T> {
            type Output = ();

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if !self.sender.channel.full.get() {
                    let bufferp = self.sender.channel.buffer.get() as *mut T;
                    unsafe { bufferp.write(ptr::read(&*self.msg)) }

                    self.sent.set(true);
                    self.sender.channel.full.set(true);

                    // wake up the receiver
                    if let Some(waker) = self.sender.channel.recv_waker.take() {
                        waker.wake();
                    }

                    Poll::Ready(())
                } else {
                    // register a waker for this sender
                    // NOTE(as_ptr) we peek into the flag of the `Option` but create no references
                    // into the `Option`'s data
                    if unsafe { (*self.sender.channel.send_waker.as_ptr()).is_none() } {
                        self.sender.channel.send_waker.set(Some(cx.waker().clone()));
                    }

                    Poll::Pending
                }
            }
        }

        impl<T> Drop for Send<'_, '_, T> {
            fn drop(&mut self) {
                if !self.sent.get() {
                    unsafe { ManuallyDrop::drop(&mut self.msg) }
                }

                // deregister the waker
                drop(self.sender.channel.send_waker.take());
            }
        }

        Send {
            msg: ManuallyDrop::new(msg),
            sender: self,
            sent: Cell::new(false),
        }
    }
}

/// The receiving side of a channel
pub struct Receiver<'c, T> {
    channel: &'c Channel<T>,
}

impl<T> Receiver<'_, T> {
    /// Receives a message from the channel
    pub fn recv<'r>(&'r mut self) -> impl Future<Output = T> + 'r {
        struct Recv<'r, 'c, T> {
            receiver: &'r Receiver<'c, T>,
        }

        impl<T> Future for Recv<'_, '_, T> {
            type Output = T;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
                if self.receiver.channel.full.get() {
                    self.receiver.channel.full.set(false);

                    let bufferp = self.receiver.channel.buffer.get() as *mut T;
                    let val = unsafe { bufferp.read() };

                    // wake up the sender
                    if let Some(waker) = self.receiver.channel.send_waker.take() {
                        waker.wake();
                    }

                    Poll::Ready(val)
                } else {
                    // register a waker for this receiver
                    // NOTE(as_ptr) we peek into the flag of the `Option` but create no references
                    // into the `Option`'s data
                    if unsafe { (*self.receiver.channel.recv_waker.as_ptr()).is_none() } {
                        self.receiver
                            .channel
                            .recv_waker
                            .set(Some(cx.waker().clone()));
                    }

                    Poll::Pending
                }
            }
        }

        impl<T> Drop for Recv<'_, '_, T> {
            fn drop(&mut self) {
                // deregister the waker
                drop(self.receiver.channel.recv_waker.take());
            }
        }

        Recv { receiver: self }
    }
}
