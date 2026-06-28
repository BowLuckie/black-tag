use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::args::quiet_args;

pub fn download_mp3(url: &str, dir: &Path, no_verbose: bool) -> Result<PathBuf> {
    let output_template = dir.join("audio");
    let status = Command::new("yt-dlp")
        .args(quiet_args(no_verbose))
        .args(["-x", "--audio-format", "mp3", "--audio-quality", "0", "-o"])
        .arg(&output_template)
        .arg(url)
        .status()?;

    if !status.success() {
        bail!("audio download failed");
    }

    Ok(dir.join("audio.mp3"))
}
