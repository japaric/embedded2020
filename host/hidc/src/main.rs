use core::str;
use std::env;

use anyhow::{anyhow, ensure};
use hidapi::HidApi;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args().skip(1).collect::<Vec<_>>(); // skip program name
    ensure!(!args.is_empty(), "expected at least one argument");

    let api = HidApi::new()?;
    let dev = api
        .device_list()
        .filter(|dev| dev.vendor_id() == consts::VID && dev.product_id() == consts::PID)
        .next()
        .ok_or_else(|| anyhow!("device not found"))?
        .open_device(&api)?;

    dev.write(args[0].as_bytes())?;
    let mut buf = [0; 64];
    let n = dev.read(&mut buf)?;
    println!("{:?}", str::from_utf8(&buf[..n]));

    Ok(())
}
