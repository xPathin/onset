use std::path::Path;

use anyhow::Result;

use crate::config::XDG_PATHS;
use crate::desktop_entry::parser::{is_valid_desktop_entry, parse_desktop_file_from_path};
use crate::model::AutostartEntry;

pub fn discover_autostart_entries() -> Result<Vec<AutostartEntry>> {
    let mut entries: Vec<AutostartEntry> = Vec::new();

    let dir = &XDG_PATHS.user_autostart;
    if !dir.exists() {
        return Ok(entries);
    }

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().map(|e| e != "desktop").unwrap_or(true) {
                continue;
            }

            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string());

            let Some(id) = id else {
                continue;
            };

            if let Some(autostart_entry) = load_autostart_entry(&path, &id) {
                entries.push(autostart_entry);
            }
        }
    }

    entries.sort_by(|a, b| a.desktop_entry.name.cmp(&b.desktop_entry.name));

    Ok(entries)
}

fn load_autostart_entry(path: &Path, id: &str) -> Option<AutostartEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    if !is_valid_desktop_entry(&content) {
        tracing::debug!("Skipping invalid desktop entry: {}", path.display());
        return None;
    }

    let (desktop_entry, raw_content) = parse_desktop_file_from_path(path).ok()?;

    Some(AutostartEntry::new(
        id.to_string(),
        path.to_path_buf(),
        desktop_entry,
        raw_content,
    ))
}
