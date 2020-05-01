//! Header functional descriptor

/// Header functional descriptor
#[allow(non_snake_case)]
pub struct Desc {
    /// Communications Devices Specification release number
    pub bcdCDC: bcdCDC,
}

/// Communications Devices specification release number
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bcdCDC {
    /// 1.10
    V11 = 0x01_10,
}

impl Desc {
    /// The size of this descriptor on the wire
    pub const SIZE: u8 = 5;

    /// Returns the wire representation of this device endpoint
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            super::CS_INTERFACE,
            super::SUBTYPE_HEADER,
            self.bcdCDC as u16 as u8,
            (self.bcdCDC as u16 >> 8) as u8,
        ]
    }
}
