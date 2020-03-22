//! Doubly-linked list

use core::{cell::Cell, ptr::NonNull};

pub struct Link<T> {
    data: T,
    next: Cell<Option<NonNull<Link<T>>>>,
    prev: Cell<Option<NonNull<Link<T>>>>,
}

impl<T> Link<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            next: Cell::new(None),
            prev: Cell::new(None),
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct DoublyLinkedList<T> {
    head: Cell<Option<NonNull<Link<T>>>>,
    // XXX we may be able to remove this; at least for `Mutex`-es
    tail: Cell<Option<NonNull<Link<T>>>>,
}

impl<T> DoublyLinkedList<T> {
    pub const fn new() -> Self {
        Self {
            head: Cell::new(None),
            tail: Cell::new(None),
        }
    }

    /// # Safety
    /// - This method erases the lifetime; the caller must ensure that `link` will be unlinked
    ///   before it's destroyed
    /// - The same `link` must not be pushed more than once into the list; it must appear at most
    ///   once
    pub unsafe fn push_front(&self, link: &Link<T>) {
        let link_ = NonNull::from(link);
        if let Some(head) = self.head.get() {
            head.as_ref().next.set(Some(link_));
            link.prev.set(Some(head));

            if self.tail.get().is_none() {
                self.tail.set(Some(head));
            }
        } else if self.tail.get().is_none() {
            self.tail.set(Some(link_));
        }

        self.head.set(Some(link_));
    }

    pub fn pop_front(&self) -> Option<NonNull<Link<T>>> {
        unsafe {
            if let Some(link) = self.head.get() {
                // unnecessary?
                // link.as_ref().prev.set(None);

                let prev = link.as_ref().prev.get();
                self.head.set(prev);

                // empty list
                if let Some(prev) = prev.as_ref() {
                    prev.as_ref().next.set(None);
                } else {
                    self.tail.set(None);
                }

                Some(link)
            } else {
                None
            }
        }
    }

    /// # Safety
    /// `link` must be part of the list
    pub unsafe fn unlink(&self, link: NonNull<Link<T>>) {
        let link = link.as_ref();
        let next = link.next.get();
        let prev = link.prev.get();

        match (prev, next) {
            // unlinked the only item in the list
            (None, None) => {
                self.head.set(None);
                self.tail.set(None);
            }

            // unlinking head
            (Some(prev), None) => {
                let new_head = prev;
                new_head.as_ref().next.set(None);
                self.head.set(Some(new_head));
            }

            // unlinking tail
            (None, Some(next)) => {
                let new_tail = next;
                new_tail.as_ref().prev.set(None);
                self.tail.set(Some(new_tail));
            }

            // unlinked something else
            (Some(prev), Some(next)) => {
                prev.as_ref().next.set(Some(next));
                next.as_ref().prev.set(Some(prev));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::ptr::NonNull;

    use super::{DoublyLinkedList, Link};

    #[test]
    fn push_pop() {
        let a = Link::new(());
        let b = Link::new(());
        let c = Link::new(());
        {
            let list = DoublyLinkedList::new();
            unsafe {
                list.push_front(&a);
            }

            assert!(a.next.get().is_none());
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&a)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));

            unsafe {
                list.push_front(&b);
            }

            assert!(b.next.get().is_none());
            assert_eq!(b.prev.get(), Some(NonNull::from(&a)));

            assert_eq!(a.next.get(), Some(NonNull::from(&b)));
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&b)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));

            unsafe {
                list.push_front(&c);
            }

            assert!(c.next.get().is_none());
            assert_eq!(c.prev.get(), Some(NonNull::from(&b)));

            assert_eq!(b.next.get(), Some(NonNull::from(&c)));
            assert_eq!(b.prev.get(), Some(NonNull::from(&a)));

            assert_eq!(a.next.get(), Some(NonNull::from(&b)));
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&c)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));

            assert_eq!(list.pop_front(), Some(NonNull::from(&c)));

            assert!(b.next.get().is_none());
            assert_eq!(b.prev.get(), Some(NonNull::from(&a)));

            assert_eq!(a.next.get(), Some(NonNull::from(&b)));
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&b)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));

            assert_eq!(list.pop_front(), Some(NonNull::from(&b)));

            assert!(a.next.get().is_none());
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&a)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));

            assert_eq!(list.pop_front(), Some(NonNull::from(&a)));

            assert!(list.head.get().is_none());
            assert!(list.tail.get().is_none());
        }
    }

    #[test]
    fn unlink() {
        let a = Link::new(());
        let b = Link::new(());
        let c = Link::new(());
        {
            let list = DoublyLinkedList::new();
            unsafe {
                list.push_front(&a);
                list.push_front(&b);
                list.push_front(&c);

                list.unlink(NonNull::from(&b));
            }

            assert!(c.next.get().is_none());
            assert_eq!(c.prev.get(), Some(NonNull::from(&a)));

            assert_eq!(a.next.get(), Some(NonNull::from(&c)));
            assert!(a.prev.get().is_none());

            assert_eq!(list.head.get(), Some(NonNull::from(&c)));
            assert_eq!(list.tail.get(), Some(NonNull::from(&a)));
        }
    }
}
