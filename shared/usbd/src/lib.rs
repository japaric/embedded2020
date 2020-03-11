//! USB device types
//!
//! # References
//!
//! - Universal Serial Bus Specification Revision 2.0

#![no_std]

use core::convert::TryFrom;

use binfmt::derive::binDebug;

#[derive(binDebug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Device-to-Host
    IN,

    /// Host-to-Device
    OUT,
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Default,
    Address,
    Configured,
}

// see table 9-4 Standard Request Codes
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum bRequest {
    GET_STATUS,
    CLEAR_FEATURE,
    SET_FEATURE,
    SET_ADDRESS,
    GET_DESCRIPTOR,
    SET_DESCRIPTOR,
    GET_CONFIGURATION,
    SET_CONFIGURATION,
    GET_INTERFACE,
    SET_INTERFACE,
    SYNCH_FRAME,
    Reserved,
    NonStandard(u8),
}

impl From<u8> for bRequest {
    fn from(byte: u8) -> Self {
        match byte {
            0 => bRequest::GET_STATUS,
            1 => bRequest::CLEAR_FEATURE,
            2 => bRequest::Reserved,
            3 => bRequest::SET_FEATURE,
            4 => bRequest::Reserved,
            5 => bRequest::SET_ADDRESS,
            6 => bRequest::GET_DESCRIPTOR,
            7 => bRequest::SET_DESCRIPTOR,
            8 => bRequest::GET_CONFIGURATION,
            9 => bRequest::SET_CONFIGURATION,
            10 => bRequest::GET_INTERFACE,
            11 => bRequest::SET_INTERFACE,
            12 => bRequest::SYNCH_FRAME,
            byte => bRequest::NonStandard(byte),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(binDebug, Clone, Copy, PartialEq)]
pub enum DescriptorType {
    DEVICE = 1,
    CONFIGURATION = 2,
    STRING = 3,
    INTERFACE = 4,
    ENDPOINT = 5,
    DEVICE_QUALIFIER = 6,
    OTHER_SPEED_CONFIGURATION = 7,
    INTERFACE_POWER = 8,
}

impl TryFrom<u8> for DescriptorType {
    type Error = ();
    fn try_from(byte: u8) -> Result<Self, ()> {
        Ok(match byte {
            1 => DescriptorType::DEVICE,
            2 => DescriptorType::CONFIGURATION,
            3 => DescriptorType::STRING,
            4 => DescriptorType::INTERFACE,
            5 => DescriptorType::ENDPOINT,
            6 => DescriptorType::DEVICE_QUALIFIER,
            7 => DescriptorType::OTHER_SPEED_CONFIGURATION,
            8 => DescriptorType::INTERFACE_POWER,
            _ => return Err(()),
        })
    }
}

/// Standard Device Descriptor
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct DeviceDesc {
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

const DEVICE_DESC_SZ: u8 = 18;

impl DeviceDesc {
    pub fn bytes(&self) -> [u8; DEVICE_DESC_SZ as usize] {
        [
            DEVICE_DESC_SZ,
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
