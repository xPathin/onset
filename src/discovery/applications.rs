use std::collections::HashSet;

use anyhow::Result;

use crate::config::XDG_PATHS;
use crate::desktop_entry::parser::{is_valid_desktop_entry, parse_desktop_file_from_path};
use crate::model::Application;
use crate::utils::binary_exists;

pub fn discover_applications() -> Result<Vec<Application>> {
    let mut applications: Vec<Application> = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for dir in XDG_PATHS.all_application_dirs() {
        if !dir.exists() {
            continue;
        }

        scan_application_dir(dir, &mut applications, &mut seen_ids)?;
    }

    applications.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(applications)
}

fn scan_application_dir(
    dir: &std::path::Path,
    applications: &mut Vec<Application>,
    seen_ids: &mut HashSet<String>,
) -> Result<()> {
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };

    for entry in read_dir.flatten() {
        let path = entry.path();

        if path.is_dir() {
            scan_application_dir(&path, applications, seen_ids)?;
            continue;
        }

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

        if seen_ids.contains(&id) {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !is_valid_desktop_entry(&content) {
            continue;
        }

        let (desktop_entry, _) = match parse_desktop_file_from_path(&path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if desktop_entry.hidden || desktop_entry.no_display {
            continue;
        }

        if let Some(ref try_exec) = desktop_entry.try_exec {
            if !binary_exists(try_exec) {
                continue;
            }
        }

        seen_ids.insert(id.clone());
        applications.push(Application::from_desktop_entry(id, &desktop_entry));
    }

    Ok(())
}
