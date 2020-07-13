use std::env;

use anyhow::{anyhow, bail, ensure};
use hidapi::HidApi;

fn main() -> Result<(), anyhow::Error> {
    let args = env::args()
        .skip(1) // skip program name
        .collect::<Vec<_>>();
    ensure!(!args.is_empty(), "expected exactly one argument");

    let api = HidApi::new()?;
    let dev = api
        .device_list()
        .filter(|dev| dev.vendor_id() == consts::VID && dev.product_id() == consts::PID)
        .next()
        .ok_or_else(|| anyhow!("device not found"))?
        .open_device(&api)?;

    let chan = args[0].parse::<u8>()?;
    if chan < 11 || chan > 26 {
        bail!("channel is out of range (`11..=26`)")
    }
    dev.write(&[chan])?;
    println!("requested channel change to channel {}", chan);

    Ok(())
}
