use std::fmt::Display;

use crate::downloader::args::DownloaderArgs;

/// represents a logger instance
#[derive(Clone, Debug)]
pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(args: &DownloaderArgs) -> Self {
        Self {
            verbose: !args.no_verbose,
        }
    }

    /// prints a yellow warning unconcerned with verbosity
    pub fn note(&self, msg: impl Display) {
        #![allow(clippy::print_stdout)]
        println!("\x1b[33mNOTE: {}\x1b[0m", msg);
    }

    /// prints a green info message concered with verbosity
    pub fn info(&self, msg: impl Display) {
        #![allow(clippy::print_stdout)]
        if self.verbose {
            println!("\x1b[34mINFO: {}\x1b[0m", msg);
        }
    }

    /// prints a green system message unconcerned with verbosity
    pub fn message(&self, msg: impl Display) {
        #![allow(clippy::print_stdout)]
        println!("\x1b[32m{}\x1b[0m", msg);
    }

    /// prints a red error message unconcerned with verbosity
    pub fn error(&self, msg: impl Display) {
        #![allow(clippy::print_stdout)]
        println!("\x1b[31mERROR: {}\x1b[0m", msg);
    }
}
