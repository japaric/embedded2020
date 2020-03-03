#![deny(warnings)]

use core::fmt;
use std::collections::BTreeMap;

use binfmt::{Level, Tag};

pub enum Node<'f> {
    F32(f32),
    Footprint(&'f str, Vec<Node<'f>>),
    I32(i32),
    Log(Level, u32, Box<Node<'f>>),
    Pointer(u32),
    U32(u32),
}

impl fmt::Display for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::F32(val) => write!(f, "{}", val),

            Node::Footprint(footprint, nodes) => {
                let args = nodes
                    .iter()
                    .map(|node| node.to_string())
                    .collect::<Vec<_>>();

                f.write_str(&dynfmt(footprint, &args))
            }

            Node::I32(val) => write!(f, "{}", val),

            Node::Log(level, ts, node) => {
                use colored::*;

                let timestamp = (*ts as f64) / 32_768.0;
                write!(f, "{:>10.6} ", timestamp)?;

                match level {
                    Level::Debug => f.write_str("DEBUG")?,
                    Level::Error => write!(f, "{}", "ERROR".red())?,
                    Level::Info => write!(f, "{} ", "INFO".green())?,
                    Level::Trace => write!(f, "{}", "TRACE".dimmed())?,
                    Level::Warn => write!(f, "{} ", "WARN".yellow())?,
                }

                write!(f, " {}", node)
            }

            Node::Pointer(val) => write!(f, "{:#010x}", val),

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

/// End of Stream
#[derive(Debug, PartialEq)]
pub struct EoS;

// NOTE assumes the stream is well-formed
pub fn parse_stream<'f>(
    bytes: &[u8],
    footprints: &BTreeMap<u64, &'f str>,
) -> Result<(Node<'f>, usize), EoS> {
    let tag = bytes.get(0).ok_or(EoS)?;
    let tag = Tag::from(*tag).expect("unreachable");
    let mut consumed = 1;

    match tag {
        Tag::F32 => {
            let bytes = bytes.get(1..5).ok_or(EoS)?;
            consumed += 4;
            let bytes = unsafe { *(bytes.as_ptr() as *const [u8; 4]) };
            let val = f32::from_le_bytes(bytes);
            Ok((Node::F32(val), consumed))
        }

        Tag::Footprint => {
            let (val, i) = leb128_decode_u32(&bytes[1..])?;
            consumed += i;
            let footprint = footprints[&val.into()];
            let mut args = vec![];
            for _ in 0..count_footprint_arguments(footprint) {
                let (arg, i) = parse_stream(&bytes[consumed..], footprints)?;
                consumed += i;
                args.push(arg);
            }
            Ok((Node::Footprint(footprint, args), consumed))
        }

        Tag::Pointer => {
            let bytes = bytes.get(1..5).ok_or(EoS)?;
            consumed += 4;
            let bytes = unsafe { *(bytes.as_ptr() as *const [u8; 4]) };
            let val = u32::from_le_bytes(bytes);
            Ok((Node::Pointer(val), consumed))
        }

        Tag::Unsigned => {
            let (val, i) = leb128_decode_u32(&bytes[1..])?;
            consumed += i;
            Ok((Node::U32(val), consumed))
        }

        Tag::Signed => todo!(),

        Tag::Debug | Tag::Error | Tag::Info | Tag::Trace | Tag::Warn => {
            let level = match tag {
                Tag::Debug => Level::Debug,
                Tag::Error => Level::Error,
                Tag::Info => Level::Info,
                Tag::Trace => Level::Trace,
                Tag::Warn => Level::Warn,
                _ => unreachable!(),
            };
            let (ts, i) = leb128_decode_u32(&bytes[1..])?;
            consumed += i;
            let (node, i) = parse_stream(&bytes[consumed..], footprints)?;
            consumed += i;
            Ok((Node::Log(level, ts, Box::new(node)), consumed))
        }
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

#[cfg(test)]
fn unzigzag(_x: u32) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
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
        let footprints = ["The answer is {}"];

        assert_eq!(
            super::parse_stream(&[Tag::Unsigned as u8, 1], &footprints)
                .unwrap()
                .0
                .to_string(),
            "1"
        );

        assert_eq!(
            super::parse_stream(
                &[Tag::Footprint as u8, 0, Tag::Unsigned as u8, 42],
                &footprints
            )
            .unwrap()
            .0
            .to_string(),
            "The answer is 42"
        );
    }

    #[ignore]
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
