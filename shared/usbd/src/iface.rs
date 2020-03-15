use crate::DescriptorType;

/// Interface Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub bLength: u8,
    // pub bDescriptorType: u8,
    pub bInterfaceNumber: u8,
    pub bAlternativeSetting: u8,
    pub bNumEndpoints: u8,
    pub bInterfaceClass: u8,
    pub bInterfaceSubClass: u8,
    pub bInterfaceProtocol: u8,
    pub iInterface: u8,
}

impl Desc {
    pub const SIZE: u8 = 9;

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
