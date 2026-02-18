#[cfg(target_os = "windows")]
fn generate_multi_size_ico() -> Result<(), Box<dyn std::error::Error>> {
    use ico::{IconDir, IconImage, ResourceType};
    use image::imageops::FilterType;
    use std::path::Path;

    let src = Path::new("assets/branding/favicon-48-modified.png");
    let out = Path::new("assets/branding/blaze.ico");

    let base = image::open(src)?.into_rgba8();

    let mut icon_dir = IconDir::new(ResourceType::Icon);
    for size in [16_u32, 24, 32, 48, 64, 128, 256] {
        let resized = image::imageops::resize(&base, size, size, FilterType::CatmullRom);
        let icon_image = IconImage::from_rgba_data(size, size, resized.into_raw());
        let entry = ico::IconDirEntry::encode(&icon_image)?;
        icon_dir.add_entry(entry);
    }

    let mut file = std::fs::File::create(out)?;
    icon_dir.write(&mut file)?;

    println!("cargo:rerun-if-changed={}", src.display());
    Ok(())
}

fn main() {
    // Guard on the *target* OS, not the host OS.
    // CARGO_CFG_TARGET_OS is set by Cargo and reflects the compilation target,
    // so cross-compiling to wasm32 on a Windows host won't trigger this block.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        #[cfg(target_os = "windows")]
        {
            if let Err(err) = generate_multi_size_ico() {
                panic!("failed to generate multi-size .ico: {err}");
            }

            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/branding/blaze.ico");
            if let Err(err) = res.compile() {
                panic!("failed to compile Windows resources: {err}");
            }
        }
    }
}
