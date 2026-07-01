#![warn(clippy::print_stdout, clippy::print_stderr)]

mod args;
mod audio;
mod file_overrides;
mod logger;
mod normalization;
mod tagging;
mod thumbnail;
mod video;

use args::DownloaderArgs;
use audio::download_mp3;
use clap::Parser;
use file_overrides::{EntryList, UrlEntry, parse_override_line, read_lines};
use logger::Logger;
use normalization::{normalize_title, normalize_url, sanitize, strip_noise};
use rayon::prelude::*;
use std::{fs, io, num::NonZero, thread};
use tempfile::TempDir;
use thumbnail::{crop_thumbnail, download_thumbnail};
use video::video_info;

use crate::downloader::tagging::VideoTagger;

#[derive(Clone, Debug)]
pub struct Downloader {
    args: DownloaderArgs,
    logger: Logger,
}

impl Downloader {
    pub fn new() -> Self {
        let args = DownloaderArgs::parse();
        let logger = Logger::new(&args);
        Self { args, logger }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let logger = &self.logger;
        let no_verbose = &self.args.no_verbose;

        let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();

        if let Some(threads) = self.args.threads {
            thread_pool_builder = thread_pool_builder.num_threads(threads.get());

            let recommended_threads: io::Result<NonZero<usize>> = thread::available_parallelism();

            match recommended_threads {
                Ok(rthread) => {
                    if rthread.get() < threads.get() {
                        logger.note(
                            format!("You have chosen a higher thread count ({}) then might be available to rayon ({})!\
                                this may lead to slowdown or other issues", threads, rthread),
                        );
                    }
                }
                Err(err) => logger.note(format!(
                    "Your os failed to report your available parallelism. be careful! error: {}",
                    err
                )),
            }
        }

        thread_pool_builder.build_global()?;

        logger.info(format!("thread count: {}", rayon::current_num_threads()));

        logger.message("Beginning download...");

        let entries = self.generate_entries()?;

        #[allow(clippy::print_stdout)]
        if !no_verbose {
            println!("entries acquired: {} entrie(s)", entries.len());
            println!("{}", entries.list_entries());
            logger.info(
                "artist and title will be determined at download time unless you are using an manual override",
            );
        }

        logger.info("Normalizing urls...");
        let normalized: Vec<UrlEntry> = entries
            .into_par_iter()
            .filter_map(|e| match normalize_url(&e.url, logger) {
                Ok(normal_url) => Some(UrlEntry {
                    url: normal_url,
                    title: e.title,
                    artist: e.artist,
                    album: e.album,
                }),

                Err(err) => {
                    logger.info(format!("failed to normalize {}: {err}", e.url));
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
            if let Err(e) = entry.process(&self.args) {
                logger.error(format!("failed: {}\n{e}", entry.url));
            }
        });

        logger.note(
            "if you wish to give the new mp3 a diffrent cover image it is recommended to use a gui like picard by MusicBrainz.",
        );

        logger.info("finished tagged all files");

        Ok(())
    }

    fn generate_entries(&self) -> Result<Vec<UrlEntry>, anyhow::Error> {
        let entries: Vec<UrlEntry> = if let Some(file) = &self.args.input_file {
            let lines = read_lines(file)?;
            if self.args.file_overrides {
                lines.iter().map(|l| parse_override_line(l)).collect()
            } else {
                lines
                    .into_iter()
                    .map(|url| UrlEntry {
                        url,
                        title: self.args.title.clone(),
                        artist: self.args.artist.clone(),
                        album: self.args.album.clone(),
                    })
                    .collect()
            }
        } else if let Some(url) = &self.args.url {
            vec![UrlEntry {
                url: url.clone(),
                title: None,
                artist: None,
                album: None,
            }]
        } else {
            anyhow::bail!("provide either a url or --input-file");
        };
        Ok(entries)
    }
}

impl UrlEntry {
    fn process(&self, args: &DownloaderArgs) -> anyhow::Result<()> {
        let entry = self;
        let url = &entry.url;
        let no_verbose = &args.no_verbose;
        let logger = Logger::new(args);
        logger.info(format!("begun processing {url}"));
        if *no_verbose {
            logger.note("errors will not be supressed")
        }
        let tmp = TempDir::new()?;
        let video_info = video_info(url)?;
        let channel = video_info.channel.clone();
        logger.info(format!("video info recived for {url}"));

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

        let album = &args.album;

        let thumb = download_thumbnail(url, tmp.path(), *no_verbose)?;
        let cover = tmp.path().join("cover.jpg");
        crop_thumbnail(&thumb, &cover, no_verbose)?;
        logger.info(format!("thumbnail aquired {url}"));
        let mp3 = download_mp3(url, tmp.path(), *no_verbose)?;
        logger.info(format!("downloaded {url}"));

        let tag_builder: VideoTagger = VideoTagger::new();

        tag_builder
            .cover(cover)
            .artist(artist)
            .title(&title)
            .album_opt(album.as_deref())
            .write_to(&mp3)?;

        fs::create_dir_all(&args.output)?;
        let final_path = args.output.join(format!("{}.mp3", sanitize(&title)));
        logger.info(format!("tagged {}", final_path.display())); // this is a lie since we
        // actually tagged a file somewhere in the temporary directory
        fs::copy(&mp3, &final_path)?;

        logger.message(format!("Done! final path: `{}`", final_path.display()));
        Ok(())
    }
}
