use id3::frame::{Picture, PictureType};
use id3::{Tag, TagLike, Version};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct VideoTagger {
    cover: Option<PathBuf>,
    artist: Option<String>,
    title: Option<String>,
    album: Option<String>,
}

impl VideoTagger {
    pub fn new() -> Self {
        Self {
            cover: None,
            artist: None,
            title: None,
            album: None,
        }
    }

    pub fn cover<P: Into<PathBuf>>(mut self, cover: P) -> Self {
        self.cover = Some(cover.into());
        self
    }

    pub fn artist<S: Into<String>>(mut self, artist: S) -> Self {
        self.artist = Some(artist.into());
        self
    }

    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn album_opt<S: Into<String>>(mut self, album: Option<S>) -> Self {
        if let Some(album_value) = album {
            self.album = Some(album_value.into());
        }
        self
    }

    pub fn write_to(self, mp3: &Path) -> id3::Result<()> {
        let mut tag = Tag::new();

        if let Some(artist) = self.artist {
            tag.set_artist(artist);
        }

        if let Some(title) = self.title {
            tag.set_title(title);
        }

        if let Some(album) = self.album {
            tag.set_album(album);
        }

        if let Some(cover) = self.cover {
            tag.add_frame(Picture {
                mime_type: "image/jpeg".into(),
                picture_type: PictureType::CoverFront,
                description: String::new(),
                data: fs::read(cover)?,
            });
        }

        tag.write_to_path(mp3, Version::Id3v24)?;
        Ok(())
    }
}
