use std::path::Path;
use std::process::Command;
use tracing::info;

pub fn clone(url: &str, ref_: &str, dest: &Path) -> anyhow::Result<()> {
    info!("Cloning {} @ {} to {:?}", url, ref_, dest);

    // 1. Clone
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(dest)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Git clone failed"));
    }

    // 2. Checkout ref
    let status = Command::new("git")
        .arg("checkout")
        .arg(ref_)
        .current_dir(dest)
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Git checkout failed"));
    }

    Ok(())
}
