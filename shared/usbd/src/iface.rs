//! Interface descriptors

use crate::DescriptorType;

/// Interface class
#[derive(Clone, Copy)]
pub enum Class {
    /// Communications and CDC Control
    Communications {
        /// Subclass
        subclass: CommunicationsSubclass,
    },
    /// CDC-Data
    CdcData,
}

impl Class {
    fn byte(&self) -> u8 {
        match self {
            Class::Communications { .. } => 0x02,
            Class::CdcData => 0x0A,
        }
    }

    fn subclass_byte(&self) -> u8 {
        match self {
            Class::Communications { subclass } => *subclass as u8,
            Class::CdcData => 0,
        }
    }
}

/// Communications subclass
#[derive(Clone, Copy)]
pub enum CommunicationsSubclass {
    /// Abstract Control Model
    Acm = 0x02,
}

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
    pub bInterfaceClass: Class,
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
            self.bInterfaceClass.byte(),
            self.bInterfaceClass.subclass_byte(),
            self.bInterfaceProtocol,
            self.iInterface,
        ]
    }
}
