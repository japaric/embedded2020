use crate::{DescriptorType, Direction};

/// Interface Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub bLength: u8,
    // pub bDescriptorType: u8,
    pub bEndpointAddress: Address,
    pub bmAttributes: bmAttributes,
    pub wMaxPacketSize: wMaxPacketSize,
    pub bInterval: u8,
}

impl Desc {
    pub const SIZE: u8 = 7;

    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        let word = self.wMaxPacketSize.word();
        [
            Self::SIZE,
            DescriptorType::ENDPOINT as u8,
            self.bEndpointAddress.byte(),
            self.bmAttributes.byte(),
            word as u8,
            (word >> 8) as u8,
            self.bInterval,
        ]
    }
}

#[derive(Clone, Copy)]
pub struct Address {
    pub number: u8,
    pub direction: Direction,
}

impl Address {
    fn byte(&self) -> u8 {
        (self.number & 0b1111) | (self.direction as u8) << 7
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bmAttributes {
    Bulk,
    Control,
    Interrupt,
    Isochronous {
        synchronization_type: SynchronizationType,
        usage_type: UsageType,
    },
}

impl bmAttributes {
    fn byte(&self) -> u8 {
        match self {
            bmAttributes::Bulk => 0b10,
            bmAttributes::Control => 0b00,
            bmAttributes::Interrupt => 0b11,
            bmAttributes::Isochronous {
                synchronization_type,
                usage_type,
            } => 0b01 | (*synchronization_type as u8) << 2 | (*usage_type as u8) << 4,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TransferType {
    Control = 0b00,
    Isochronous = 0b01,
    Bulk = 0b10,
    Interrupt = 0b11,
}

#[derive(Clone, Copy)]
pub enum SynchronizationType {
    NoSynchronization = 0b00,
    Asynchronous = 0b01,
    Adaptive = 0b10,
    Synchronous = 0b11,
}

#[derive(Clone, Copy)]
pub enum UsageType {
    DataEndpoint = 0b00,
    FeedbackEndpoint = 0b01,
    ImplicitFeedbackDataEndpoint = 0b10,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum wMaxPacketSize {
    BulkControl {
        /// Must be less than `1 << 11`
        size: u16,
    },

    IsochronousInterrupt {
        /// Must be less than `1 << 11`
        size: u16,
        /// Must be less than `4`
        transactions_per_microframe: u8,
    },
}

impl wMaxPacketSize {
    fn word(&self) -> u16 {
        match self {
            wMaxPacketSize::BulkControl { size } => *size & ((1 << 11) - 1),

            wMaxPacketSize::IsochronousInterrupt {
                size,
                transactions_per_microframe,
            } => (*size & ((1 << 11) - 1)) | (u16::from(*transactions_per_microframe & 0b11) << 11),
        }
    }
}
