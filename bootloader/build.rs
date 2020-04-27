use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = &PathBuf::from(env::var("OUT_DIR")?);
    let pkg_name = env::var("CARGO_PKG_NAME")?;
    let target = env::var("TARGET")?;

    // place the pre-compiled assembly somewhere the linker can find it
    if target.starts_with("thumb") {
        fs::copy(
            format!("bin/{}.a", target),
            out_dir.join(format!("lib{}.a", pkg_name)),
        )?;
        println!("cargo:rustc-link-lib=static={}", pkg_name);

        println!("cargo:rustc-link-search={}", out_dir.display());
    }

    // put the linker script somewhere the linker can find it
    fs::copy("link.x", out_dir.join("link.x"))?;
    println!("cargo:rustc-link-search={}", out_dir.display());
    Ok(())
}
