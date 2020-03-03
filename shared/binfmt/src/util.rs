use core::{hint, slice};

use crate::binWrite;

pub fn binfmt_u32(tag: u8, val: u32, f: &mut impl binWrite) {
    if val < 128 {
        // fast path
        f.write(&[tag, val as u8])
    } else {
        // NOTE(unsafe) avoid all panicking branches in optimized code
        unsafe {
            let mut buf: [u8; 6] = crate::uninitialized();
            buf[0] = tag;
            let n = leb128_encode_u32(
                val,
                &mut *(buf.as_mut_ptr().add(1) as *mut _),
            );
            f.write(slice::from_raw_parts(buf.as_ptr(), n + 1));
        }
    }
}

const CONTINUE: u8 = 1 << 7;

unsafe fn leb128_encode_u32(mut val: u32, buf: &mut [u8; 5]) -> usize {
    for (i, slot) in buf.iter_mut().enumerate() {
        *slot = (val & 0x7f) as u8;
        val >>= 7;
        if val == 0 {
            return i + 1;
        } else {
            *slot |= CONTINUE;
        }
    }
    hint::unreachable_unchecked()
}

pub fn zigzag(x: i32) -> u32 {
    ((x << 1) ^ (x >> 31)) as u32
}

#[cfg(test)]
mod tests {
    #[test]
    fn leb128() {
        let mut buf = [0; 5];

        let i = unsafe { super::leb128_encode_u32(1, &mut buf) };
        assert_eq!(buf[..i], [1]);

        let i = unsafe { super::leb128_encode_u32(127, &mut buf) };
        assert_eq!(buf[..i], [127]);

        let i = unsafe { super::leb128_encode_u32(128, &mut buf) };
        assert_eq!(buf[..i], [super::CONTINUE, 1]);

        let i = unsafe { super::leb128_encode_u32(255, &mut buf) };
        assert_eq!(buf[..i], [0x7f | super::CONTINUE, 1]);

        let i = unsafe { super::leb128_encode_u32(16383, &mut buf) };
        assert_eq!(buf[..i], [0x7f | super::CONTINUE, 0x7f]);

        let i = unsafe { super::leb128_encode_u32(16384, &mut buf) };
        assert_eq!(buf[..i], [super::CONTINUE, super::CONTINUE, 1]);

        let i = unsafe { super::leb128_encode_u32(u32::max_value(), &mut buf) };
        assert_eq!(
            buf[..i],
            [
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0b1111
            ]
        );
    }

    #[test]
    fn zigzag() {
        assert_eq!(super::zigzag(0), 0);
        assert_eq!(super::zigzag(-1), 0b001);
        assert_eq!(super::zigzag(1), 0b010);
        assert_eq!(super::zigzag(-2), 0b011);
        assert_eq!(super::zigzag(2), 0b100);
        assert_eq!(super::zigzag(i32::min_value()), 0xffffffff);
        assert_eq!(super::zigzag(i32::max_value()), 0xfffffffe);
    }
}
