#![deny(warnings)]

use core::fmt::{self, Write as _};
use std::collections::BTreeMap;

use binfmt::{Level, Tag};

/// Log message
pub struct Message<'f> {
    pub level: Level,
    pub timestamp: u32,
    pub footprint: &'f str,
    pub args: Vec<Node<'f>>,
}

impl fmt::Display for Message<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colored::*;

        let timestamp = (self.timestamp as f64) / 1_000_000.;
        write!(f, "{:>10.6} ", timestamp)?;

        match self.level {
            Level::Debug => f.write_str("DEBUG ")?,
            Level::Error => write!(f, "{} ", "ERROR".red())?,
            Level::Info => write!(f, "{}  ", "INFO".green())?,
            Level::Trace => write!(f, "{} ", "TRACE".dimmed())?,
            Level::Warn => write!(f, "{}  ", "WARN".yellow())?,
        }

        let args = self
            .args
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>();

        f.write_str(&dynfmt(self.footprint, &args))
    }
}

pub enum Node<'f> {
    Bytes(Vec<u8>),
    CLikeEnum(&'f str, u8),
    F32(f32),
    Footprint(&'f str, Vec<Node<'f>>),
    I32(i32),
    Pointer(u32),
    Register(&'f str, u32),
    U32(u32),
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Bytes(bytes) => {
                f.write_str("[")?;
                let mut first = true;
                for byte in bytes {
                    if first {
                        first = false;
                    } else {
                        f.write_str(", ")?;
                    }
                    write!(f, "{:#04x}", byte)?;
                }
                f.write_str("]")
            }

            Node::CLikeEnum(list, discr) => {
                write!(f, "{}", list.split(',').nth((*discr).into()).unwrap())
            }

            Node::F32(val) => write!(f, "{}", val),

            Node::Footprint(footprint, nodes) => {
                let args = nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>();

                f.write_str(&dynfmt(footprint, &args))
            }

            Node::I32(val) => write!(f, "{}", val),

            Node::Pointer(val) => write!(f, "{:#010x}", val),

            Node::Register(footprint, val) => f.write_str(&dynfmt_register(footprint, *val)),

            Node::U32(val) => write!(f, "{}", val),
        }
    }
}

fn dynfmt(footprint: &str, args: &[String]) -> String {
    let mut s = String::new();
    let mut args = args.iter();
    let mut chars = footprint.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let next = chars.peek();

            if next == Some(&'}') {
                // argument
                chars.next();
                s.push_str(args.next().expect("unreachable"));
            } else if next == Some(&'{') {
                // escaped brace
                chars.next();
                s.push('{');
            } else {
                unreachable!()
            }
        } else if c == '}' {
            let next = chars.peek();

            if next == Some(&'}') {
                // escaped brace
                chars.next();
                s.push('}');
            } else {
                unreachable!()
            }
        } else {
            s.push(c);
        }
    }
    s
}

// NOTE assumes the footprint is well-formed
fn dynfmt_register(footprint: &str, val: u32) -> String {
    let mut s = String::new();
    let mut chars = footprint.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c == '{' {
            let next = chars.peek().map(|ci| ci.1);

            if next == Some('{') {
                // escaped brace
                chars.next();
                s.push('{');
            } else {
                let end = footprint[start..].find('}').unwrap() + start;

                for _ in start..end {
                    // skip this argument in the next `while let` iteration
                    drop(chars.next());
                }

                // NOTE(+1) skips the left brace (`{`)
                let bitfield = &footprint[start + 1..end];

                if bitfield.contains(':') {
                    // range
                    let mut parts = bitfield.splitn(2, ':');
                    let start = parts.next().unwrap().parse::<u8>().unwrap();
                    let end = parts.next().unwrap().parse::<u8>().unwrap();
                    let width = end - start;

                    let bits = if width != 32 {
                        (val >> start) & ((1 << (end - start)) - 1)
                    } else {
                        val
                    };

                    // TODO improve formatting
                    // - use leading zeros, e.g. 2-bit fields should be
                    //   formatted as `0b00` and `0b01`
                    // - use underscores to split long sequences in groups of 4,
                    //   e.g. `0b10_0101` and `0xaa_bbcc`
                    if width < 8 {
                        write!(&mut s, "{:#b}", bits).unwrap()
                    } else {
                        write!(&mut s, "{:#x}", bits).unwrap()
                    }
                } else {
                    // single bit
                    let i = bitfield.parse::<u8>().unwrap();

                    if val & (1 << i) == 0 {
                        s.push('0')
                    } else {
                        s.push('1')
                    }
                }
            }
        } else if c == '}' {
            let next = chars.peek().map(|ci| ci.1);

            if next == Some('}') {
                // escaped brace
                chars.next();
                s.push('}');
            } else {
                unreachable!()
            }
        } else {
            s.push(c);
        }
    }
    s
}

/// End of Stream
#[derive(Debug, PartialEq)]
pub struct EoS;

// NOTE assumes the stream is well-formed
pub fn parse_message<'f>(
    bytes: &[u8],
    footprints: &BTreeMap<u64, &'f str>,
) -> Result<(Message<'f>, usize), EoS> {
    let tag = bytes.get(0).ok_or(EoS)?;
    let tag = Tag::from(*tag).expect("unreachable");
    let mut consumed = 1;

    let level = match tag {
        Tag::Debug => Level::Debug,
        Tag::Error => Level::Error,
        Tag::Info => Level::Info,
        Tag::Trace => Level::Trace,
        Tag::Warn => Level::Warn,
        _ => unreachable!(),
    };
    let (timestamp, i) = leb128_decode_u32(&bytes[consumed..])?;
    consumed += i;
    let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
    consumed += i;
    let footprint = footprints[&val.into()];
    let mut args = vec![];
    for _ in 0..count_footprint_arguments(footprint) {
        let (arg, i) = parse_node(&bytes[consumed..], footprints)?;
        consumed += i;
        args.push(arg);
    }

    Ok((
        Message {
            level,
            timestamp,
            footprint,
            args,
        },
        consumed,
    ))
}

// NOTE assumes the stream is well-formed
pub fn parse_node<'f>(
    bytes: &[u8],
    footprints: &BTreeMap<u64, &'f str>,
) -> Result<(Node<'f>, usize), EoS> {
    let tag = bytes.get(0).ok_or(EoS)?;
    let tag = Tag::from(*tag).expect("unreachable");
    let mut consumed = 1;

    match tag {
        Tag::Bytes => {
            let (len, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            let len = len as usize;
            let bytes = bytes.get(consumed..consumed + len as usize).ok_or(EoS)?;
            consumed += len;
            Ok((Node::Bytes(bytes.to_owned()), consumed))
        }

        Tag::CLikeEnum => {
            let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            let footprint = footprints[&val.into()];
            let discr = bytes.get(consumed).ok_or(EoS)?;
            consumed += 1;
            Ok((Node::CLikeEnum(footprint, *discr), consumed))
        }

        Tag::F32 => {
            let bytes = bytes.get(consumed..consumed + 4).ok_or(EoS)?;
            consumed += 4;
            let bytes = unsafe { *(bytes.as_ptr() as *const [u8; 4]) };
            let val = f32::from_le_bytes(bytes);
            Ok((Node::F32(val), consumed))
        }

        Tag::Footprint => {
            let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            let footprint = footprints[&val.into()];
            let mut args = vec![];
            for _ in 0..count_footprint_arguments(footprint) {
                let (arg, i) = parse_node(&bytes[consumed..], footprints)?;
                consumed += i;
                args.push(arg);
            }
            Ok((Node::Footprint(footprint, args), consumed))
        }

        Tag::Pointer => {
            let bytes = bytes.get(consumed..consumed + 4).ok_or(EoS)?;
            consumed += 4;
            let bytes = unsafe { *(bytes.as_ptr() as *const [u8; 4]) };
            let val = u32::from_le_bytes(bytes);
            Ok((Node::Pointer(val), consumed))
        }

        Tag::Unsigned => {
            let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            Ok((Node::U32(val), consumed))
        }

        Tag::Register => {
            let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            let footprint = footprints[&val.into()];
            let width = get_register_width(footprint);

            let p = bytes.get(consumed..consumed + width).ok_or(EoS)?.as_ptr();
            consumed += width;

            let val = unsafe {
                if width == 1 {
                    *p as u32
                } else if width == 2 {
                    u16::from_le_bytes(*(p as *const [u8; 2])).into()
                } else if width <= 4 {
                    u32::from_le_bytes(*(p as *const [u8; 4]))
                } else {
                    unreachable!()
                }
            };
            Ok((Node::Register(footprint, val), consumed))
        }

        Tag::Signed => {
            let (val, i) = leb128_decode_u32(&bytes[consumed..])?;
            consumed += i;
            Ok((Node::I32(unzigzag(val)), consumed))
        },

        Tag::Debug | Tag::Error | Tag::Info | Tag::Trace | Tag::Warn => unreachable!(),
    }
}

// NOTE assumes the footprint is well-formed
fn count_footprint_arguments(footprint: &str) -> usize {
    let mut chars = footprint.chars().peekable();
    let mut n = 0;
    while let Some(c) = chars.next() {
        if c == '{' {
            let next = chars.peek();
            if next == Some(&'}') {
                n += 1;
                chars.next();
            } else if next == Some(&'{') {
                // escaped brace
                chars.next();
            }
        }
    }
    n
}

// NOTE assumes the footprint is well-formed
// NOTE in bytes
fn get_register_width(footprint: &str) -> usize {
    let mut chars = footprint.char_indices().peekable();
    while let Some((start, c)) = chars.next() {
        if c == '{' {
            let next = chars.peek().map(|ci| ci.1);
            if next == Some('{') {
                // escaped brace
                chars.next();
            } else {
                // bitfield
                let end = footprint[start..].find('}').unwrap() + start;
                // NOTE(+1) skip the left brace (`{`)
                let bitfield = &footprint[start + 1..end];

                let highest_bit = if bitfield.contains(':') {
                    // range
                    let mut parts = bitfield.splitn(2, ':');
                    drop(parts.next());
                    parts.next().unwrap().parse::<usize>().unwrap()
                } else {
                    // individual bit
                    bitfield.parse().unwrap()
                };

                return if highest_bit <= 8 {
                    1
                } else if highest_bit <= 16 {
                    2
                } else if highest_bit <= 32 {
                    4
                } else {
                    unreachable!()
                };
            }
        }
    }
    unreachable!()
}

const CONTINUE: u8 = 1 << 7;

fn leb128_decode_u32(bytes: &[u8]) -> Result<(u32, usize), EoS> {
    let mut val = 0;
    for (i, byte) in bytes.iter().enumerate() {
        val |= u32::from(*byte & !CONTINUE) << (7 * i);

        if *byte & CONTINUE == 0 {
            return Ok((val, i + 1));
        }
    }

    Err(EoS)
}

fn unzigzag(x: u32) -> i32 {
    use core::ops::Neg as _;

    (((x & 1) as i32).neg() as u32 ^ (x >> 1)) as i32
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use binfmt::Tag;

    #[test]
    fn leb128() {
        assert_eq!(super::leb128_decode_u32(&[1]), Ok((1, 1)));
        assert_eq!(super::leb128_decode_u32(&[127]), Ok((127, 1)));

        assert_eq!(
            super::leb128_decode_u32(&[super::CONTINUE, 1]),
            Ok((128, 2))
        );
        assert_eq!(
            super::leb128_decode_u32(&[0x7f | super::CONTINUE, 1]),
            Ok((255, 2))
        );
        assert_eq!(
            super::leb128_decode_u32(&[0x7f | super::CONTINUE, 0x7f]),
            Ok((16383, 2))
        );

        assert_eq!(
            super::leb128_decode_u32(&[super::CONTINUE, super::CONTINUE, 1]),
            Ok((16384, 3))
        );

        assert_eq!(
            super::leb128_decode_u32(&[
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0x7f | super::CONTINUE,
                0b1111
            ]),
            Ok((u32::max_value(), 5))
        );
    }

    #[test]
    fn parse_and_format() {
        let mut footprints = BTreeMap::new();
        footprints.insert(0, "The answer is {}");

        assert_eq!(
            super::parse_node(&[Tag::Unsigned as u8, 1], &footprints)
                .unwrap()
                .0
                .to_string(),
            "1"
        );

        assert_eq!(
            super::parse_node(
                &[Tag::Footprint as u8, 0, Tag::Unsigned as u8, 42],
                &footprints
            )
            .unwrap()
            .0
            .to_string(),
            "The answer is 42"
        );
    }

    #[test]
    fn unzigzag() {
        assert_eq!(super::unzigzag(0), 0);
        assert_eq!(super::unzigzag(0b001), -1);
        assert_eq!(super::unzigzag(0b010), 1);
        assert_eq!(super::unzigzag(0b011), -2);
        assert_eq!(super::unzigzag(0b100), 2);
        assert_eq!(super::unzigzag(0xffffffff), i32::min_value());
        assert_eq!(super::unzigzag(0xfffffffe), i32::max_value());
    }
}
