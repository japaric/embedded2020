use crate::DescriptorType;

/// Standard Device Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Desc {
    // pub blength: u8,
    // pub bDescriptorType: u8,
    pub bcdUSB: bcdUSB,
    pub bDeviceClass: u8,
    pub bDeviceSubClass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize0: bMaxPacketSize0,
    pub idVendor: u16,
    pub idProduct: u16,
    pub bcdDevice: u16,
    pub iManufacturer: u8,
    pub iProduct: u8,
    pub iSerialNumber: u8,
    pub bNumConfigurations: u8,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bcdUSB {
    /// 2.0
    V20 = 0x0200,
    // TODO(?) other versions
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum bMaxPacketSize0 {
    B8 = 8,
    B16 = 16,
    B32 = 32,
    B64 = 64,
}

impl Desc {
    pub const SIZE: u8 = 18;

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
