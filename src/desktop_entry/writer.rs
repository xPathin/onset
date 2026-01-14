use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

use super::parser::escape_value;
use super::types::{CreateOptions, DesktopEntry};
use crate::operations::delay::{unwrap_delay, wrap_with_delay};

pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().context("Invalid path: no parent directory")?;

    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create directory: {}", parent.display()))?;

    let temp_path = path.with_extension("tmp");

    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file: {}", temp_path.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to temp file: {}", temp_path.display()))?;

    file.sync_all()
        .with_context(|| format!("Failed to sync temp file: {}", temp_path.display()))?;

    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "Failed to rename {} to {}",
            temp_path.display(),
            path.display()
        )
    })?;

    Ok(())
}

pub fn write_desktop_entry(
    path: &Path,
    name: &str,
    exec: &str,
    options: &CreateOptions,
) -> Result<()> {
    let mut content = String::new();
    content.push_str("[Desktop Entry]\n");
    content.push_str("Type=Application\n");
    content.push_str(&format!("Name={}\n", escape_value(name)));

    let final_exec = if options.delay_seconds > 0 {
        wrap_with_delay(exec, options.delay_seconds)
    } else {
        exec.to_string()
    };
    content.push_str(&format!("Exec={}\n", final_exec));

    if let Some(ref icon) = options.icon {
        content.push_str(&format!("Icon={}\n", icon));
    }
    if let Some(ref comment) = options.comment {
        content.push_str(&format!("Comment={}\n", escape_value(comment)));
    }
    if options.terminal {
        content.push_str("Terminal=true\n");
    }
    if !options.only_show_in.is_empty() {
        content.push_str(&format!("OnlyShowIn={};\n", options.only_show_in.join(";")));
    }
    if !options.not_show_in.is_empty() {
        content.push_str(&format!("NotShowIn={};\n", options.not_show_in.join(";")));
    }
    if options.hidden {
        content.push_str("Hidden=true\n");
    }

    write_atomic(path, &content)
}

pub fn update_desktop_entry_content(
    content: &str,
    entry: &DesktopEntry,
    delay_seconds: Option<u32>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_desktop_entry = false;
    let mut keys_written: std::collections::HashSet<&str> = std::collections::HashSet::new();

    let (base_exec, _) = unwrap_delay(&entry.exec);
    let final_exec = match delay_seconds {
        Some(d) if d > 0 => wrap_with_delay(&base_exec, d),
        _ => base_exec,
    };

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[Desktop Entry]" {
            in_desktop_entry = true;
            lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with('[') {
            in_desktop_entry = false;
            lines.push(line.to_string());
            continue;
        }

        if !in_desktop_entry {
            lines.push(line.to_string());
            continue;
        }

        if let Some((key, _)) = trimmed.split_once('=') {
            let key = key.trim();
            match key {
                "Name" => {
                    lines.push(format!("Name={}", escape_value(&entry.name)));
                    keys_written.insert("Name");
                }
                "Exec" => {
                    lines.push(format!("Exec={}", final_exec));
                    keys_written.insert("Exec");
                }
                "Icon" => {
                    if let Some(ref icon) = entry.icon {
                        lines.push(format!("Icon={}", icon));
                    }
                    keys_written.insert("Icon");
                }
                "Comment" => {
                    if let Some(ref comment) = entry.comment {
                        lines.push(format!("Comment={}", escape_value(comment)));
                    }
                    keys_written.insert("Comment");
                }
                "Hidden" => {
                    if entry.hidden {
                        lines.push("Hidden=true".to_string());
                    }
                    keys_written.insert("Hidden");
                }
                "Terminal" => {
                    if entry.terminal {
                        lines.push("Terminal=true".to_string());
                    }
                    keys_written.insert("Terminal");
                }
                "OnlyShowIn" => {
                    if !entry.only_show_in.is_empty() {
                        lines.push(format!("OnlyShowIn={};", entry.only_show_in.join(";")));
                    }
                    keys_written.insert("OnlyShowIn");
                }
                "NotShowIn" => {
                    if !entry.not_show_in.is_empty() {
                        lines.push(format!("NotShowIn={};", entry.not_show_in.join(";")));
                    }
                    keys_written.insert("NotShowIn");
                }
                _ => {
                    lines.push(line.to_string());
                }
            }
        } else {
            lines.push(line.to_string());
        }
    }

    lines.join("\n") + "\n"
}

pub fn sanitize_id(id: &str) -> String {
    let sanitized: String = id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Trim leading/trailing underscores and collapse multiple underscores
    let trimmed: String = sanitized
        .trim_matches('_')
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    // If result is empty or only dots/dashes, generate a fallback
    if trimmed.is_empty() || trimmed.chars().all(|c| c == '.' || c == '-') {
        format!(
            "autostart_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        )
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("my-app"), "my-app");
        assert_eq!(sanitize_id("My App"), "My_App");
        assert_eq!(sanitize_id("app@2.0"), "app_2.0");
    }

    #[test]
    fn test_sanitize_id_edge_cases() {
        // All illegal chars - should get fallback
        let result = sanitize_id("@#$%");
        assert!(result.starts_with("autostart_"));

        // Empty string - should get fallback
        let result = sanitize_id("");
        assert!(result.starts_with("autostart_"));

        // Only spaces - should get fallback
        let result = sanitize_id("   ");
        assert!(result.starts_with("autostart_"));

        // Mixed with valid chars
        assert_eq!(sanitize_id("  my app  "), "my_app");
        assert_eq!(sanitize_id("@my@app@"), "my_app");
    }
}
