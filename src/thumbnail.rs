use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Result, bail};

pub fn download_thumbnail(url: &str, dir: &Path) -> Result<PathBuf> {
    let output_template = dir.join("thumb");
    let status = Command::new("yt-dlp")
        .args(["--skip-download", "--write-thumbnail", "-o"])
        .arg(&output_template)
        .arg(url)
        .status()?;
    if !status.success() {
        bail!("thumbnail download failed");
    }
    for ext in ["webp", "jpg", "jpeg", "png"] {
        let path = dir.join(format!("thumb.{ext}"));
        if path.exists() {
            return Ok(path);
        }
    }
    bail!("thumbnail not found")
}

pub fn crop_thumbnail(input: &Path, output: &Path) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args(["-y", "-i"])
        .arg(input)
        .args(["-vf", "crop='min(iw,ih)':'min(iw,ih)'"])
        .arg(output)
        .status()?;
    if !status.success() {
        bail!("thumbnail crop failed");
    }
    Ok(())
}
