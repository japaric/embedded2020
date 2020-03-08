use crate::{binDebug, binWrite, util, Tag};

impl binDebug for f32 {
    fn fmt(&self, f: &mut impl binWrite) {
        f.write_byte(Tag::F32 as u8);
        f.write(&self.to_le_bytes());
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
        f.write_byte(Tag::Signed as u8);
        f.leb128_write(util::zigzag(*self));
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
        f.write_byte(Tag::Unsigned as u8);
        f.leb128_write(*self);
    }
}

#[cfg(target_pointer_width = "32")]
impl<T> binDebug for *const T {
    fn fmt(&self, f: &mut impl binWrite) {
        f.write_byte(Tag::Pointer as u8);
        f.write(&(*self as u32).to_le_bytes());
    }
}

#[cfg(target_pointer_width = "32")]
impl<T> binDebug for *mut T {
    fn fmt(&self, f: &mut impl binWrite) {
        <*const T as binDebug>::fmt(&(*self as *const T), f)
    }
}

#[cfg(target_pointer_width = "32")]
impl binDebug for [u8] {
    fn fmt(&self, f: &mut impl binWrite) {
        f.write_byte(Tag::Bytes as u8);
        f.leb128_write(self.len() as u32);
        f.write(self);
    }
}
