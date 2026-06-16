use std::process::Command;

pub fn normalize_title(raw: &str, artist: Option<&str>, strip_until: Option<char>) -> String {
    let mut title = raw.trim().to_string();

    title = strip_noise(&title);
    title = split_best_guess(&title, artist);

    if let Some(c) = strip_until {
        title = strip_until_last(&title, c);
    }

    title
}

fn split_best_guess(s: &str, artist: Option<&str>) -> String {
    let parts: Vec<&str> = s
        .split(['|', '-', ':'])
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    if parts.len() == 1 {
        return parts[0].to_string();
    }

    if let Some(artist) = artist {
        let artist_l = artist.to_lowercase();
        if let Some(first) = parts.iter().find(|p| p.to_lowercase() != artist_l) {
            return (*first).to_string();
        }
    }

    parts.last().unwrap_or(&"").to_string()
}

pub fn strip_noise(s: &str) -> String {
    let mut s = s.to_string();

    let patterns = [
        "(official audio)",
        "(official video)",
        "(official music video)",
        "[official audio]",
        "[official video]",
        "[lyrics]",
        "(lyrics)",
    ];

    let lower = s.to_lowercase();

    for p in patterns {
        let lp = p.to_lowercase();

        if lower.contains(&lp) {
            // remove original-case version safely
            s = remove_case_insensitive(&s, p);
        }
    }

    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn remove_case_insensitive(input: &str, pattern: &str) -> String {
    let lower = input.to_lowercase();
    let pat = pattern.to_lowercase();

    if let Some(pos) = lower.find(&pat) {
        let mut result = input.to_string();
        let end = pos + pattern.len();
        result.replace_range(pos..end, "");
        return result;
    }

    input.to_string()
}

pub fn normalize_url(url: &str) -> anyhow::Result<String> {
    let output = Command::new("yt-dlp")
        .args(["--no-playlist", "--print", "%(webpage_url)s", url])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("yt-dlp failed to normalize '{url}': {stderr}");
    }
    let cleaned = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if cleaned.is_empty() {
        anyhow::bail!("yt-dlp returned empty output for '{url}'");
    }

    eprintln!("url normalized {url} --> {cleaned}");
    Ok(cleaned)
}

fn strip_until_last(s: &str, c: char) -> String {
    match s.rfind(c) {
        Some(idx) if idx + 1 < s.len() => s[idx + 1..].trim().to_string(),
        _ => s.to_string(),
    }
}
