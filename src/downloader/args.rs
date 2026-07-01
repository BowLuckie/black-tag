use clap::Parser;
use std::{num::NonZero, path::PathBuf};

#[derive(Parser, Clone, Debug)]
pub struct DownloaderArgs {
    /// YouTube URL
    pub url: Option<String>,

    /// Override artist
    #[arg(long)]
    pub artist: Option<String>,

    /// Override title
    #[arg(long)]
    pub title: Option<String>,

    /// Output directory default .
    #[arg(short, long, default_value = ".")]
    pub output: PathBuf,

    /// File to read urls from
    #[arg(long)]
    pub input_file: Option<PathBuf>,

    /// strips anything before this character for titles. does not affect artist
    #[arg(long)]
    pub strip_until: Option<char>,

    /// parse --input-file as "url | title | artist" per line
    #[arg(long)]
    pub file_overrides: bool,

    /// Supress yt-dlp output and process info. wont supress errors
    #[arg(long)]
    pub no_verbose: bool,

    /// max numbers of threads to use to process. defaults to letting rayon decide
    #[arg(short, long)]
    pub threads: Option<NonZero<usize>>,
}
