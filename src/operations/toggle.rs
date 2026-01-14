use std::path::Path;

use anyhow::{Context, Result};

use crate::desktop_entry::writer::write_atomic;

pub fn set_entry_enabled_by_path(path: &Path, enabled: bool) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read entry: {}", path.display()))?;

    let mut lines: Vec<String> = Vec::new();
    let mut in_desktop_entry = false;
    let mut hidden_written = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[Desktop Entry]" {
            in_desktop_entry = true;
            lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with('[') {
            if in_desktop_entry && !enabled && !hidden_written {
                lines.push("Hidden=true".to_string());
                hidden_written = true;
            }
            in_desktop_entry = false;
            lines.push(line.to_string());
            continue;
        }

        if in_desktop_entry {
            if let Some((key, _)) = trimmed.split_once('=') {
                if key.trim() == "Hidden" {
                    if !enabled {
                        lines.push("Hidden=true".to_string());
                        hidden_written = true;
                    }
                    continue;
                }
            }
        }

        lines.push(line.to_string());
    }

    if in_desktop_entry && !enabled && !hidden_written {
        lines.push("Hidden=true".to_string());
    }

    let new_content = lines.join("\n") + "\n";

    write_atomic(path, &new_content)
        .with_context(|| format!("Failed to update entry: {}", path.display()))?;

    tracing::info!(
        "Set autostart entry at {} to {}",
        path.display(),
        if enabled { "enabled" } else { "disabled" }
    );

    Ok(())
}
