use std::path::{Path, PathBuf};
use std::fs;
use std::time::SystemTime;

pub fn get_file_size_str(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

pub fn get_file_modified_time(path: &Path) -> Option<String> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()
        .map(|time| {
            let datetime = chrono::DateTime::<chrono::Local>::from(time);
            datetime.format("%Y-%m-%d %H:%M").to_string()
        })
}

pub fn get_file_icon(path: &Path) -> &'static str {
    if path.is_dir() {
        "📁"
    } else {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => "📄",
            Some("rs") | Some("js") | Some("py") | Some("html") | Some("css") => "📝",
            Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "🖼️",
            Some("mp4") | Some("avi") | Some("mkv") => "🎬",
            Some("mp3") | Some("wav") | Some("flac") => "🎵",
            Some("pdf") => "📕",
            Some("zip") | Some("rar") | Some("7z") => "📦",
            Some("exe") | Some("msi") => "⚙️",
            _ => "📄",
        }
    }
}

pub fn is_hidden_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}