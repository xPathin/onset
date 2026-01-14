use std::path::Path;

use anyhow::{Context, Result};

use super::types::DesktopEntry;

pub fn parse_desktop_file(content: &str) -> Result<DesktopEntry> {
    let mut entry = DesktopEntry::default();
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => entry.name = unescape_value(value),
                "Exec" => entry.exec = value.to_string(),
                "Icon" => entry.icon = Some(value.to_string()),
                "Comment" => entry.comment = Some(unescape_value(value)),
                "Hidden" => entry.hidden = value.eq_ignore_ascii_case("true"),
                "Terminal" => entry.terminal = value.eq_ignore_ascii_case("true"),
                "NoDisplay" => entry.no_display = value.eq_ignore_ascii_case("true"),
                "TryExec" => entry.try_exec = Some(value.to_string()),
                "OnlyShowIn" => {
                    entry.only_show_in = value
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "NotShowIn" => {
                    entry.not_show_in = value
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "Categories" => {
                    entry.categories = value
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "Keywords" => {
                    entry.keywords = value
                        .split(';')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                _ => {}
            }
        }
    }

    Ok(entry)
}

pub fn parse_desktop_file_from_path(path: &Path) -> Result<(DesktopEntry, String)> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read desktop file: {}", path.display()))?;

    let entry = parse_desktop_file(&content)?;
    Ok((entry, content))
}

pub fn is_valid_desktop_entry(content: &str) -> bool {
    let mut has_type = false;
    let mut type_is_application = false;
    let mut has_name = false;
    let mut has_exec = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }

        if line.starts_with('[') {
            in_desktop_entry = false;
            continue;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "Type" => {
                    has_type = true;
                    type_is_application = value.trim() == "Application";
                }
                "Name" => has_name = !value.trim().is_empty(),
                "Exec" => has_exec = !value.trim().is_empty(),
                _ => {}
            }
        }
    }

    has_type && type_is_application && has_name && has_exec
}

fn unescape_value(value: &str) -> String {
    value
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\")
}

pub fn escape_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_entry() {
        let content = r#"[Desktop Entry]
Type=Application
Name=Test App
Exec=/usr/bin/test
Icon=test-icon
Comment=A test application
"#;
        let entry = parse_desktop_file(content).unwrap();
        assert_eq!(entry.name, "Test App");
        assert_eq!(entry.exec, "/usr/bin/test");
        assert_eq!(entry.icon, Some("test-icon".to_string()));
        assert_eq!(entry.comment, Some("A test application".to_string()));
    }

    #[test]
    fn test_parse_hidden_entry() {
        let content = r#"[Desktop Entry]
Type=Application
Name=Hidden App
Exec=/usr/bin/hidden
Hidden=true
"#;
        let entry = parse_desktop_file(content).unwrap();
        assert!(entry.hidden);
    }

    #[test]
    fn test_parse_only_show_in() {
        let content = r#"[Desktop Entry]
Type=Application
Name=GNOME App
Exec=/usr/bin/gnome-app
OnlyShowIn=GNOME;Unity;
"#;
        let entry = parse_desktop_file(content).unwrap();
        assert_eq!(entry.only_show_in, vec!["GNOME", "Unity"]);
    }

    #[test]
    fn test_is_valid_desktop_entry() {
        let valid = r#"[Desktop Entry]
Type=Application
Name=Test
Exec=/usr/bin/test
"#;
        assert!(is_valid_desktop_entry(valid));

        let missing_type = r#"[Desktop Entry]
Name=Test
Exec=/usr/bin/test
"#;
        assert!(!is_valid_desktop_entry(missing_type));

        let wrong_type = r#"[Desktop Entry]
Type=Link
Name=Test
Exec=/usr/bin/test
"#;
        assert!(!is_valid_desktop_entry(wrong_type));
    }
}
