//! Communication Device Class functional descriptors

pub mod acm;
pub mod call;
pub mod header;
pub mod union;

const CS_INTERFACE: u8 = 0x24;

const SUBTYPE_HEADER: u8 = 0x00;
const SUBTYPE_CALL: u8 = 0x01;
const SUBTYPE_ACM: u8 = 0x02;
const SUBTYPE_UNION: u8 = 0x06;
