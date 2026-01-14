use std::path::PathBuf;

use crate::config::get_current_desktop;
use crate::desktop_entry::{DesktopEntry, EffectiveState};
use crate::operations::delay::{get_delay, unwrap_delay};
use crate::utils::binary_exists;

#[derive(Debug, Clone)]
pub struct AutostartEntry {
    pub id: String,
    pub path: PathBuf,
    pub desktop_entry: DesktopEntry,
    pub effective_state: EffectiveState,
    pub raw_content: String,
}

impl AutostartEntry {
    pub fn new(
        id: String,
        path: PathBuf,
        desktop_entry: DesktopEntry,
        raw_content: String,
    ) -> Self {
        let mut entry = Self {
            id,
            path,
            desktop_entry,
            effective_state: EffectiveState::Enabled,
            raw_content,
        };
        entry.effective_state = entry.compute_effective_state(&get_current_desktop());
        entry
    }

    pub fn compute_effective_state(&self, current_desktop: &[String]) -> EffectiveState {
        if self.desktop_entry.hidden {
            return EffectiveState::Disabled;
        }

        if let Some(ref try_exec) = self.desktop_entry.try_exec {
            if !binary_exists(try_exec) {
                return EffectiveState::TryExecFailed;
            }
        }

        if !self.desktop_entry.only_show_in.is_empty()
            && !current_desktop
                .iter()
                .any(|d| self.desktop_entry.only_show_in.contains(d))
        {
            return EffectiveState::EnvironmentExcluded;
        }

        if current_desktop
            .iter()
            .any(|d| self.desktop_entry.not_show_in.contains(d))
        {
            return EffectiveState::EnvironmentExcluded;
        }

        EffectiveState::Enabled
    }

    pub fn delay_seconds(&self) -> Option<u32> {
        get_delay(&self.desktop_entry.exec)
    }

    pub fn base_exec(&self) -> String {
        let (base, _) = unwrap_delay(&self.desktop_entry.exec);
        base
    }
}
