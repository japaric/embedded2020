//! Interface descriptors

use crate::DescriptorType;

/// Interface Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub bLength: u8,
    // pub bDescriptorType: u8,
    /// Interface number
    pub bInterfaceNumber: u8,
    /// Alternative setting
    pub bAlternativeSetting: u8,
    /// Number of endpoints
    pub bNumEndpoints: u8,
    /// Interface class
    pub bInterfaceClass: u8,
    /// Interface subclass
    pub bInterfaceSubClass: u8,
    /// Interface protocol
    pub bInterfaceProtocol: u8,
    /// Interface string descriptor index
    pub iInterface: u8,
}

impl Desc {
    /// The size of this descriptor in bytes
    pub const SIZE: u8 = 9;

    /// Returns the byte representation of this descriptor
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            DescriptorType::INTERFACE as u8,
            self.bInterfaceNumber,
            self.bAlternativeSetting,
            self.bNumEndpoints,
            self.bInterfaceClass,
            self.bInterfaceSubClass,
            self.bInterfaceProtocol,
            self.iInterface,
        ]
    }
}
