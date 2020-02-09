use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    // put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    fs::copy("link.x", out.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out.display());
    Ok(())
}
