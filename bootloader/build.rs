use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = &PathBuf::from(env::var("OUT_DIR")?);

    // put the linker script somewhere the linker can find it
    fs::copy("link.x", out_dir.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());
    Ok(())
}
