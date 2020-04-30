//! Device descriptor

use crate::DescriptorType;

/// Standard Device Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub blength: u8,
    // pub bDescriptorType: u8,
    /// USB specification release version
    pub bcdUSB: bcdUSB,
    /// Device class
    pub bDeviceClass: u8,
    /// Device subclass
    pub bDeviceSubClass: u8,
    /// Device protocol
    pub bDeviceProtocol: u8,
    /// Maximum packet size
    pub bMaxPacketSize0: bMaxPacketSize0,
    /// Vendor ID
    pub idVendor: u16,
    /// Product ID
    pub idProduct: u16,
    /// Device release number
    pub bcdDevice: u16,
    /// Manufacturer string index
    pub iManufacturer: u8,
    /// Product string index
    pub iProduct: u8,
    /// Serial number string index
    pub iSerialNumber: u8,
    /// Number of configurations
    pub bNumConfigurations: u8,
}

/// USB specification release version
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bcdUSB {
    /// 2.0
    V20 = 0x0200,
    // TODO(?) other versions
}

/// Maximum packet size
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bMaxPacketSize0 {
    /// 8 bytes
    B8 = 8,
    /// 16 bytes
    B16 = 16,
    /// 32 bytes
    B32 = 32,
    /// 64 bytes
    B64 = 64,
}

impl Desc {
    /// The size of this descriptor on the wire
    pub const SIZE: u8 = 18;

    /// Returns the wire representation of this device endpoint
    pub fn bytes(&self) -> [u8; Self::SIZE as usize] {
        [
            Self::SIZE,
            DescriptorType::DEVICE as u8,
            self.bcdUSB as u16 as u8,
            (self.bcdUSB as u16 >> 8) as u8,
            self.bDeviceClass,
            self.bDeviceSubClass,
            self.bDeviceProtocol,
            self.bMaxPacketSize0 as u8,
            self.idVendor as u8,
            (self.idVendor >> 8) as u8,
            self.idProduct as u8,
            (self.idProduct >> 8) as u8,
            self.bcdDevice as u8,
            (self.bcdDevice >> 8) as u8,
            self.iManufacturer,
            self.iProduct,
            self.iSerialNumber,
            self.bNumConfigurations,
        ]
    }
}
