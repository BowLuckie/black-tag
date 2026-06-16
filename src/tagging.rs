use anyhow::Result;
use id3::frame::{Picture, PictureType};
use id3::{Tag, TagLike, Version};
use std::fs;
use std::path::Path;

pub fn tag_mp3(mp3: &Path, cover: &Path, artist: &str, title: &str) -> Result<()> {
    let mut tag = Tag::new();
    tag.set_artist(artist);
    tag.set_title(title);
    tag.add_frame(Picture {
        mime_type: "image/jpeg".into(),
        picture_type: PictureType::CoverFront,
        description: String::new(),
        data: fs::read(cover)?,
    });
    tag.write_to_path(mp3, Version::Id3v24)?;
    Ok(())
}
