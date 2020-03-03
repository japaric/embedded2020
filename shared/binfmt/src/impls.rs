use crate::{binDebug, binWrite, util, Tag};

impl binDebug for f32 {
    fn fmt(&self, f: &mut impl binWrite) {
        let bytes = self.to_le_bytes();
        let bytes = [Tag::F32 as u8, bytes[0], bytes[1], bytes[2], bytes[3]];
        f.write(&bytes);
    }
}

impl binDebug for i8 {
    fn fmt(&self, f: &mut impl binWrite) {
        <i32 as binDebug>::fmt(&((*self).into()), f)
    }
}

impl binDebug for i16 {
    fn fmt(&self, f: &mut impl binWrite) {
        <i32 as binDebug>::fmt(&((*self).into()), f)
    }
}

impl binDebug for i32 {
    fn fmt(&self, f: &mut impl binWrite) {
        util::binfmt_u32(Tag::Signed as u8, util::zigzag(*self), f)
    }
}

impl binDebug for u8 {
    fn fmt(&self, f: &mut impl binWrite) {
        <u32 as binDebug>::fmt(&((*self).into()), f)
    }
}

impl binDebug for u16 {
    fn fmt(&self, f: &mut impl binWrite) {
        <u32 as binDebug>::fmt(&((*self).into()), f)
    }
}

impl binDebug for u32 {
    fn fmt(&self, f: &mut impl binWrite) {
        util::binfmt_u32(Tag::Unsigned as u8, *self, f)
    }
}

#[cfg(target_pointer_width = "32")]
impl<T> binDebug for *const T {
    fn fmt(&self, f: &mut impl binWrite) {
        let bytes = (*self as u32).to_le_bytes();
        let bytes = [Tag::Pointer as u8, bytes[0], bytes[1], bytes[2], bytes[3]];
        f.write(&bytes);
    }
}

#[cfg(target_pointer_width = "32")]
impl<T> binDebug for *mut T {
    fn fmt(&self, f: &mut impl binWrite) {
        <*const T as binDebug>::fmt(&(*self as *const T), f)
    }
}
