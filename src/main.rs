#![warn(clippy::print_stdout, clippy::print_stderr)]

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
use std::fmt::Display;
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

    message("Beginning download...");

    let no_verbose = &args.no_verbose;

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

    info(
        format!("entries acquired: {} entries", entries.len()),
        no_verbose,
    );

    #[allow(clippy::print_stdout)]
    if !no_verbose {
        println!("{entries:#?}");
    }

    info("Normalizing urls...", no_verbose);
    let normalized: Vec<UrlEntry> = entries
        .into_par_iter()
        .filter_map(|e| match normalize_url(&e.url, no_verbose) {
            Ok(n) => Some(UrlEntry {
                url: n,
                title: e.title,
                artist: e.artist,
            }),
            Err(err) => {
                info(format!("failed to normalize {}: {err}", e.url), no_verbose);
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
            error(format!("failed: {}\n{e}", entry.url));
        }
    });

    info("finished tagged all files", no_verbose);

    Ok(())
}

fn process_one(args: &Args, entry: &UrlEntry) -> anyhow::Result<()> {
    let url = &entry.url;
    let no_verbose = &args.no_verbose;
    info(format!("begun processing {url}"), no_verbose);
    let tmp = TempDir::new()?;
    let video_info = video_info(url, *no_verbose)?;
    let channel = video_info.channel.clone();
    info(format!("video info recived for {url}"), no_verbose);

    let artist = entry
        .artist
        .clone()
        .or_else(|| args.artist.clone())
        .unwrap_or_else(|| {
            video_info
                .uploader
                .or(channel)
                .unwrap_or_else(|| "Unknown Artist".into())
        });

    let mut title = entry
        .title
        .clone()
        .or_else(|| args.title.clone())
        .unwrap_or(video_info.title);

    title = normalize_title(&title, Some(artist.as_str()), args.strip_until);
    title = strip_noise(&title);

    let thumb = download_thumbnail(url, tmp.path(), *no_verbose)?;
    let cover = tmp.path().join("cover.jpg");
    crop_thumbnail(&thumb, &cover, no_verbose)?;
    info(format!("thumbnail aquired {url}"), no_verbose);
    let mp3 = download_mp3(url, tmp.path(), *no_verbose)?;
    info(format!("downloaded {url}"), no_verbose);
    tag_mp3(&mp3, &cover, &artist, &title)?;
    fs::create_dir_all(&args.output)?;
    let final_path = args.output.join(format!("{}.mp3", sanitize(&title)));
    info(format!("tagged {}", final_path.display()), no_verbose); // this is a lie since we
    // actually tagged a file somewhere in the temporary directory
    fs::copy(&mp3, &final_path)?;

    note(
        "if you wish to give the new mp3 a diffrent cover image it is recommended to use a gui like picard by MusicBrainz.",
    );
    message(format!("Done! final path: `{}`", final_path.display()));
    Ok(())
}

fn read_lines(path: &Path) -> anyhow::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

/// prints a yellow warning unconcerned with verbosity
pub fn note(msg: &str) {
    #![allow(clippy::print_stdout)]
    println!("\x1b[33mNOTE: {}\x1b[0m", msg);
}

/// prints a green info message concered with verbosity
pub fn info(msg: impl Display, no_verbose: &bool) {
    #![allow(clippy::print_stdout)]
    if !no_verbose {
        println!("\x1b[32mINFO: {}\x1b[0m", msg);
    }
}

/// prints a green system message unconcerned with verbosity
pub fn message(msg: impl Display) {
    #![allow(clippy::print_stdout)]
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn error(msg: impl Display) {
    #![allow(clippy::print_stdout)]
    println!("\x1b[31mERROR: {}\x1b[0m", msg);
}
