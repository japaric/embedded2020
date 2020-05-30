use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = &PathBuf::from(env::var("OUT_DIR")?);

    descs(&out_dir)?;

    // put the linker script somewhere the linker can find it
    fs::copy("link.x", out_dir.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());

    Ok(())
}

// generate USB descriptors
fn descs(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    use core::num::NonZeroU8;

    use quote::quote;
    use usb2::{
        configuration::{self, bmAttributes},
        device::{self, bMaxPacketSize0},
    };

    const PACKET_SIZE: bMaxPacketSize0 = bMaxPacketSize0::B64;
    const CONFIG_VAL: u8 = 1;

    let device_desc = device::Descriptor {
        bDeviceClass: 0,
        bDeviceSubClass: 0,
        bDeviceProtocol: 0,

        bMaxPacketSize0: bMaxPacketSize0::B64,
        bNumConfigurations: NonZeroU8::new(1).unwrap(),
        bcdDevice: 0x01_00,
        iManufacturer: None,
        iProduct: None,
        iSerialNumber: None,
        idProduct: consts::PID,
        idVendor: consts::VID,
    };

    fn full_config_desc() -> Vec<u8> {
        let mut bytes = vec![];

        let config_desc = configuration::Descriptor {
            bConfigurationValue: NonZeroU8::new(CONFIG_VAL).unwrap(),
            bMaxPower: 250, // 500 mA
            bNumInterfaces: 0,
            bmAttributes: bmAttributes {
                remote_wakeup: false,
                self_powered: true,
            },
            iConfiguration: None,
            // NOTE this will be fixed at the end of this function
            wTotalLength: 0,
        };

        bytes.extend_from_slice(&config_desc.bytes());

        let total_length = bytes.len();
        assert!(
            total_length <= u16::max_value() as usize,
            "configuration descriptor is too long"
        );

        bytes[2] = total_length as u8;
        bytes[3] = (total_length >> 8) as u8;

        bytes
    }

    let max_packet_size0 = PACKET_SIZE as u8;
    let ddb = device_desc.bytes();
    let ddl = ddb.len();
    let cdb = full_config_desc();
    let cdl = cdb.len();
    fs::write(
        out_dir.join("descs.rs"),
        quote!(
            const CONFIG_VAL: core::num::NonZeroU8 = unsafe {
                core::num::NonZeroU8::new_unchecked(#CONFIG_VAL)
            };
            const MAX_PACKET_SIZE0: u8 = #max_packet_size0;
            static CONFIG_DESC: [u8; #cdl] = [#(#cdb,)*];
            static DEVICE_DESC: [u8; #ddl] = [#(#ddb,)*];
        )
        .to_string(),
    )?;

    Ok(())
}
