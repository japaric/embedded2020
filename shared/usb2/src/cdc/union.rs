//! Union Interface functional descriptor

/// Union Interface functional descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // bFunctionLength: u8,
    // bDescriptorType: u8,
    // bDescriptorSubtype: u8,
    /// Controlling interface
    pub bControlInterface: u8,
    /// Subordinate interface
    pub bSubordinateInterface0: u8,
}

impl Desc {
    /// Size of this descriptor on the wire
    pub const SIZE: u8 = 5;

    /// Returns the wire representation of this device endpoint
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            super::CS_INTERFACE,
            super::SUBTYPE_UNION,
            self.bControlInterface,
            self.bSubordinateInterface0,
        ]
    }
}
