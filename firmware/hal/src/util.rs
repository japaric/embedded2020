use core::ops;

#[repr(align(4))]
pub(crate) struct Align4<T>(pub T);

impl<T> ops::Deref for Align4<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for Align4<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
