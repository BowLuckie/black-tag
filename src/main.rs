mod args;
mod audio;
mod normalization;
mod tagging;
mod thumbnail;
mod video;

use args::Args;
use args::sanitize;
use audio::download_mp3;
use clap::Parser;
use normalization::normalize_url;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use tagging::tag_mp3;
use tempfile::TempDir;
use thumbnail::{crop_thumbnail, download_thumbnail};
use video::video_info;

use crate::normalization::normalize_title;
use crate::normalization::strip_noise;

fn main() -> anyhow::Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()?;

    let args = Args::parse();

    let urls: Vec<String> = if let Some(file) = &args.input_file {
        read_lines(file)?
    } else if let Some(url) = &args.url {
        vec![url.clone()]
    } else {
        anyhow::bail!("provide either a url or --input-file");
    };

    eprintln!("urls acquired: {urls:#?}");

    eprintln!("Normalizing urls...");
    let mut seen = std::collections::HashSet::new();
    let urls: Vec<String> = urls
        .into_iter()
        .filter_map(|u| match normalize_url(&u) {
            Ok(n) => Some(n),
            Err(e) => {
                eprintln!("failed to normalize {u}: {e}");
                None
            }
        })
        .filter(|n| seen.insert(n.clone()))
        .collect();

    urls.par_iter().for_each(|url| {
        if let Err(e) = process_one(&args, url) {
            eprintln!("failed: {url}\n{e}");
        }
    });

    eprintln!("finished tagged all files");

    Ok(())
}

fn process_one(args: &Args, url: &str) -> anyhow::Result<()> {
    eprintln!("begun processing {url}");
    let tmp = TempDir::new()?;
    let info = video_info(url)?;
    let (_uploader, channel) = (info.uploader.clone(), info.channel.clone());
    eprintln!("video info recived for {url}");

    let artist = args.artist.clone().unwrap_or_else(|| {
        info.uploader
            .or(channel)
            .unwrap_or_else(|| "Unknown Artist".into())
    });

    let mut title = args.title.clone().unwrap_or(info.title);

    title = normalize_title(
        &title,
        args.artist.as_deref().or(Some(artist.as_str())),
        args.strip_until,
    );

    title = strip_noise(&title);

    let thumb = download_thumbnail(url, tmp.path())?;
    let cover = tmp.path().join("cover.jpg");
    crop_thumbnail(&thumb, &cover)?;
    eprintln!("thumbnail aquired {url}");

    let mp3 = download_mp3(url, tmp.path())?;
    eprintln!("downloaded {url}");
    tag_mp3(&mp3, &cover, &artist, &title)?;
    eprintln!("tagged {url}");

    fs::create_dir_all(&args.output)?;

    let final_path = args.output.join(format!("{}.mp3", sanitize(&title)));

    fs::copy(&mp3, &final_path)?;

    println!("{}", final_path.display());
    Ok(())
}

fn read_lines(path: &Path) -> anyhow::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}
