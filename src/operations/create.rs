use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::config::XDG_PATHS;
use crate::desktop_entry::CreateOptions;
use crate::desktop_entry::writer::{sanitize_id, write_desktop_entry};

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

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    XDG_PATHS
        .user_autostart
        .join(format!("{}_{}.desktop", base_id, timestamp))
}
