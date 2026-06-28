use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Args {
    /// YouTube URL
    pub url: Option<String>,
    /// Override artist
    #[arg(long)]
    pub artist: Option<String>,
    /// Override title
    #[arg(long)]
    pub title: Option<String>,
    /// Output directory
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
}

pub fn sanitize(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

pub fn quiet_args(no_verbose: bool) -> &'static [&'static str] {
    if no_verbose {
        &["--quiet", "--no-warnings"]
    } else {
        &[]
    }
}
