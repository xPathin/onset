use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::config::XDG_PATHS;
use crate::desktop_entry::writer::{sanitize_id, write_desktop_entry};
use crate::desktop_entry::CreateOptions;

pub fn create_autostart_entry(
    id: &str,
    name: &str,
    exec: &str,
    options: CreateOptions,
) -> Result<PathBuf> {
    let sanitized_id = sanitize_id(id);
    if sanitized_id.is_empty() {
        bail!("Invalid entry ID: {}", id);
    }

    // Find a unique filename by adding suffix if needed
    let path = find_unique_path(&sanitized_id);

    write_desktop_entry(&path, name, exec, &options)
        .with_context(|| format!("Failed to create autostart entry: {}", path.display()))?;

    tracing::info!("Created autostart entry: {}", path.display());

    Ok(path)
}

fn find_unique_path(base_id: &str) -> PathBuf {
    let base_path = XDG_PATHS
        .user_autostart
        .join(format!("{}.desktop", base_id));

    if !base_path.exists() {
        return base_path;
    }

    for i in 1..1000 {
        let suffixed_path = XDG_PATHS
            .user_autostart
            .join(format!("{}_{}.desktop", base_id, i));
        if !suffixed_path.exists() {
            return suffixed_path;
        }
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    XDG_PATHS
        .user_autostart
        .join(format!("{}_{}.desktop", base_id, timestamp))
}
