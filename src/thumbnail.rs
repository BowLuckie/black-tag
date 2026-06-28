use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::args::quiet_args;

pub fn download_thumbnail(url: &str, dir: &Path, no_verbose: bool) -> Result<PathBuf> {
    let output_template = dir.join("thumb");
    let status = Command::new("yt-dlp")
        .args(quiet_args(no_verbose))
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

pub fn crop_thumbnail(input: &Path, output: &Path, no_verbose: &bool) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args(shush_ffmpg(no_verbose))
        .args(["-y", "-i"])
        .arg(input)
        .args([
            "-vf",
            "scale='if(gt(a,1),-2,1080)':'if(gt(a,1),1080,-2)',crop=1080:1080",
        ])
        .arg(output)
        .status()?;
    if !status.success() {
        bail!("thumbnail crop failed");
    }
    Ok(())
}

fn shush_ffmpg(no_verbose: &bool) -> &'static [&'static str] {
    if *no_verbose {
        &["-loglevel", "quiet"]
    } else {
        &[]
    }
}
