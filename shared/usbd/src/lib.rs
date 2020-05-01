//! USB device types
//!
//! # References
//!
//! - Universal Serial Bus Specification Revision 2.0

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use core::convert::TryFrom;

use binfmt::derive::binDebug;

pub mod cdc;
pub mod config;
pub mod device;
pub mod ep;
pub mod iface;

/// USB direction (from the point of view of the host)
#[derive(binDebug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Host-to-Device
    OUT = 0,

    /// Device-to-Host
    IN = 1,
}

/// USB device state
#[derive(Clone, Copy, PartialEq)]
pub enum State {
    /// Default state
    Default,
    /// Addressed state
    Address,
    /// Configured state
    Configured {
        /// Configuration value
        configuration: u8,
    },
}

/// Standard Request Code
// see table 9-4 Standard Request Codes
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq)]
pub enum bRequest {
    /// Get status
    GET_STATUS,
    /// Clear feature
    CLEAR_FEATURE,
    /// Set feature
    SET_FEATURE,
    /// Set address
    SET_ADDRESS,
    /// Get descriptor
    GET_DESCRIPTOR,
    /// Set descriptor
    SET_DESCRIPTOR,
    /// Get configuration
    GET_CONFIGURATION,
    /// Set configuration
    SET_CONFIGURATION,
    /// Get interface
    GET_INTERFACE,
    /// Set interface
    SET_INTERFACE,
    /// Synch frame
    SYNCH_FRAME,
    /// Reserved request code
    Reserved,
    /// Non-standard request code
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

/// Descriptor type
#[allow(non_camel_case_types)]
#[derive(binDebug, Clone, Copy, PartialEq)]
pub enum DescriptorType {
    /// Device descriptor
    DEVICE = 1,
    /// Configuration descriptor
    CONFIGURATION = 2,
    /// String descriptor
    STRING = 3,
    /// Interface descriptor
    INTERFACE = 4,
    /// Endpoint descriptor
    ENDPOINT = 5,
    /// Device qualifier descriptor
    DEVICE_QUALIFIER = 6,
    /// Other speed configuration descriptor
    OTHER_SPEED_CONFIGURATION = 7,
    /// Interface power descriptor
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
