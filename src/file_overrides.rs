#[derive(Debug)]
pub struct UrlEntry {
    pub(crate) url: String,
    pub(crate) title: Option<String>,
    pub(crate) artist: Option<String>,
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
