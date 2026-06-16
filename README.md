# black-tag

A small CLI tool that downloads YouTube videos, extracts audio as MP3, embeds metadata, and attaches a square-cropped thumbnail as album art.

## Features

* Download audio from YouTube via `yt-dlp`
* Convert to MP3 using `ffmpeg`
* Extract video metadata (title, uploader)
* Optional overrides for artist and title
* Download and crop thumbnail to square
* Embed ID3 tags with cover art
* Batch mode via input file
* Simple filename sanitization

---

## Requirements

* `yt-dlp`
* `ffmpeg`

Install dependencies:

arch
```bash
sudo pacman -S yt-dlp ffmpeg
```

---


## Disclaimer

This tool is intended for downloading and processing media that you have the legal right to access. The author does not endorse or encourage copyright infringement.

Users are solely responsible for ensuring compliance with applicable copyright laws, platform terms of service, and local regulations when using this software.
