use std::path::PathBuf;

use once_cell::sync::Lazy;

pub static XDG_PATHS: Lazy<XdgPaths> = Lazy::new(XdgPaths::new);

#[derive(Debug, Clone)]
pub struct XdgPaths {
    pub user_autostart: PathBuf,
    pub user_applications: PathBuf,
    pub system_applications: Vec<PathBuf>,
}

impl XdgPaths {
    pub fn new() -> Self {
        let xdg = xdg::BaseDirectories::new();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        let home_path = PathBuf::from(&home);

        let config_home = xdg.config_home.unwrap_or_else(|| home_path.join(".config"));
        let data_home = xdg
            .data_home
            .unwrap_or_else(|| home_path.join(".local/share"));

        Self {
            user_autostart: config_home.join("autostart"),
            user_applications: data_home.join("applications"),
            system_applications: xdg
                .data_dirs
                .iter()
                .map(|p| p.join("applications"))
                .collect(),
        }
    }

    pub fn all_application_dirs(&self) -> Vec<&PathBuf> {
        let mut dirs = vec![&self.user_applications];
        dirs.extend(&self.system_applications);
        dirs
    }
}

impl Default for XdgPaths {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_current_desktop() -> Vec<String> {
    std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
