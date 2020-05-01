//! Call Management functional descriptor

/// Call Management functional descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // bFunctionLength: u8,
    // bDescriptorType: u8,
    // bDescriptorSubtype: u8,
    /// Capabilities
    pub bmCapabilities: Capabilities,
    /// Interface of the Data Class interface
    pub bDataInterface: u8,
}

/// Capabilities
#[derive(Clone, Copy)]
pub struct Capabilities {
    /// Device handles call management itself
    pub call_management: bool,
    /// Device can send/receive call management information over a Data Class interface
    pub data_class: bool,
}

impl Capabilities {
    fn byte(&self) -> u8 {
        let mut byte = 0;
        if self.call_management {
            byte |= 1 << 0;
        }
        if self.data_class {
            byte |= 1 << 1;
        }
        byte
    }
}

impl Desc {
    /// Size of this descriptor on the wire
    pub const SIZE: u8 = 5;

    /// Returns the wire representation of this device endpoint
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            super::CS_INTERFACE,
            super::SUBTYPE_CALL,
            self.bmCapabilities.byte(),
            self.bDataInterface,
        ]
    }
}
