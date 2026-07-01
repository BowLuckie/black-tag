use crate::downloader::Downloader;

mod downloader;

fn main() {
    let mut downloader = Downloader::new();
    downloader
        .run()
        .unwrap_or_else(|err| println!("the downloader failed to run! {}", err));
}
