use anyhow::{Context, Result};

use crate::desktop_entry::writer::{update_desktop_entry_content, write_atomic};
use crate::desktop_entry::EntryChanges;
use crate::model::AutostartEntry;
use crate::operations::delay::unwrap_delay;

pub fn edit_autostart_entry(entry: &AutostartEntry, changes: EntryChanges) -> Result<()> {
    let mut updated_entry = entry.desktop_entry.clone();

    if let Some(name) = changes.name {
        updated_entry.name = name;
    }
    if let Some(exec) = changes.exec {
        updated_entry.exec = exec;
    }
    if let Some(comment) = changes.comment {
        updated_entry.comment = Some(comment);
    }
    if let Some(icon) = changes.icon {
        updated_entry.icon = Some(icon);
    }
    if let Some(hidden) = changes.hidden {
        updated_entry.hidden = hidden;
    }
    if let Some(terminal) = changes.terminal {
        updated_entry.terminal = terminal;
    }
    if let Some(only_show_in) = changes.only_show_in {
        updated_entry.only_show_in = only_show_in;
    }
    if let Some(not_show_in) = changes.not_show_in {
        updated_entry.not_show_in = not_show_in;
    }

    let delay = changes.delay_seconds.or_else(|| {
        let (_, existing_delay) = unwrap_delay(&entry.desktop_entry.exec);
        existing_delay
    });

    let new_content = update_desktop_entry_content(&entry.raw_content, &updated_entry, delay);

    write_atomic(&entry.path, &new_content)
        .with_context(|| format!("Failed to save entry: {}", entry.path.display()))?;

    tracing::info!("Updated autostart entry: {}", entry.path.display());

    Ok(())
}
