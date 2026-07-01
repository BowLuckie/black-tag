use std::path::Path;

#[derive(Debug)]
pub struct UrlEntry {
    pub url: String,
    pub title: Option<String>,
    pub artist: Option<String>,
}

pub trait EntryList {
    fn list_entries(&self) -> String;
}

impl EntryList for [UrlEntry] {
    fn list_entries(&self) -> String {
        let mut result = String::new();

        for entry in self {
            let entry_as_string = format!(
                "{} - {} ({})",
                entry.title.as_deref().unwrap_or("?"),
                entry.artist.as_deref().unwrap_or("?"),
                entry.url,
            );

            result.push_str(&entry_as_string);
            result.push('\n');
        }

        result
    }
}

pub fn read_lines(path: &Path) -> anyhow::Result<Vec<String>> {
    Ok(std::fs::read_to_string(path)?
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

pub fn parse_override_line(line: &str) -> UrlEntry {
    let parts: Vec<&str> = line.split('|').map(|p| p.trim()).collect();
    let url = parts.first().copied().unwrap_or("").to_string();
    let title = parts
        .get(1)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let artist = parts
        .get(2)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    UrlEntry { url, title, artist }
}
