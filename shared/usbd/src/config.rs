use crate::DescriptorType;

/// Configuration Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub blength: u8,
    // pub bDescriptorType: u8,
    pub wTotalLength: u16,
    pub bNumInterfaces: u8,
    pub bConfigurationValue: u8,
    pub iConfiguration: u8,
    pub bmAttributes: bmAttributes,
    pub bMaxPower: u8,
}

impl Desc {
    pub const SIZE: u8 = 9;

    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            DescriptorType::CONFIGURATION as u8,
            self.wTotalLength as u8,
            (self.wTotalLength >> 8) as u8,
            self.bNumInterfaces,
            self.bConfigurationValue,
            self.iConfiguration,
            (1 << 7)
                | if self.bmAttributes.self_powered {
                    1 << 6
                } else {
                    0
                }
                | if self.bmAttributes.remote_wakeup {
                    1 << 5
                } else {
                    0
                },
            self.bMaxPower,
        ]
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct bmAttributes {
    pub self_powered: bool,
    pub remote_wakeup: bool,
}
