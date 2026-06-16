use anyhow::{Result, bail};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub uploader: Option<String>,
    pub channel: Option<String>,
}

pub fn video_info(url: &str) -> Result<VideoInfo> {
    let output = Command::new("yt-dlp")
        .args(["--dump-single-json", url])
        .output()?;

    if !output.status.success() {
        bail!("yt-dlp metadata lookup failed");
    }

    Ok(serde_json::from_slice(&output.stdout)?)
}

