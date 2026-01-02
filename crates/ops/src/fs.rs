use fs_extra::dir::CopyOptions;
use std::path::Path;

pub fn ensure_dir(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn copy(src: &Path, dest: &Path) -> anyhow::Result<()> {
    let options = CopyOptions::new(); // Default options
    fs_extra::dir::copy(src, dest, &options)?;
    Ok(())
}
