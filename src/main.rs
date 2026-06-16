mod args;
mod audio;
mod file_overrides;
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

use crate::file_overrides::UrlEntry;
use crate::file_overrides::parse_override_line;
use crate::normalization::normalize_title;
use crate::normalization::strip_noise;

fn main() -> anyhow::Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()?;

    let args = Args::parse();

    let entries: Vec<UrlEntry> = if let Some(file) = &args.input_file {
        let lines = read_lines(file)?;
        if args.file_overrides {
            lines.iter().map(|l| parse_override_line(l)).collect()
        } else {
            lines
                .into_iter()
                .map(|url| UrlEntry {
                    url,
                    title: None,
                    artist: None,
                })
                .collect()
        }
    } else if let Some(url) = &args.url {
        vec![UrlEntry {
            url: url.clone(),
            title: None,
            artist: None,
        }]
    } else {
        anyhow::bail!("provide either a url or --input-file");
    };

    eprintln!("entries acquired: {} entries", entries.len());
    eprintln!("{entries:#?}");

    eprintln!("Normalizing urls...");
    let normalized: Vec<UrlEntry> = entries
        .into_par_iter()
        .filter_map(|e| match normalize_url(&e.url) {
            Ok(n) => Some(UrlEntry {
                url: n,
                title: e.title,
                artist: e.artist,
            }),
            Err(err) => {
                eprintln!("failed to normalize {}: {err}", e.url);
                None
            }
        })
        .collect();

    let mut seen = std::collections::HashSet::new();
    let entries: Vec<UrlEntry> = normalized
        .into_iter()
        .filter(|e| seen.insert(e.url.clone()))
        .collect();

    entries.par_iter().for_each(|entry| {
        if let Err(e) = process_one(&args, entry) {
            eprintln!("failed: {}\n{e}", entry.url);
        }
    });

    eprintln!("finished tagged all files");

    Ok(())
}

fn process_one(args: &Args, entry: &UrlEntry) -> anyhow::Result<()> {
    let url = &entry.url;
    eprintln!("begun processing {url}");
    let tmp = TempDir::new()?;
    let info = video_info(url)?;
    let (_uploader, channel) = (info.uploader.clone(), info.channel.clone());
    eprintln!("video info recived for {url}");

    let artist = entry
        .artist
        .clone()
        .or_else(|| args.artist.clone())
        .unwrap_or_else(|| {
            info.uploader
                .or(channel)
                .unwrap_or_else(|| "Unknown Artist".into())
        });

    let mut title = entry
        .title
        .clone()
        .or_else(|| args.title.clone())
        .unwrap_or(info.title);

    title = normalize_title(&title, Some(artist.as_str()), args.strip_until);
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
