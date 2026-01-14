#[derive(Debug, Clone, Default)]
pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub hidden: bool,
    pub terminal: bool,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
    pub try_exec: Option<String>,
    pub no_display: bool,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveState {
    Enabled,
    Disabled,
    EnvironmentExcluded,
    TryExecFailed,
}

impl std::fmt::Display for EffectiveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectiveState::Enabled => write!(f, "Enabled"),
            EffectiveState::Disabled => write!(f, "Disabled"),
            EffectiveState::EnvironmentExcluded => write!(f, "Environment Excluded"),
            EffectiveState::TryExecFailed => write!(f, "TryExec Failed"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CreateOptions {
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub delay_seconds: u32,
    pub terminal: bool,
    pub only_show_in: Vec<String>,
    pub not_show_in: Vec<String>,
    pub hidden: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EntryChanges {
    pub name: Option<String>,
    pub exec: Option<String>,
    pub comment: Option<String>,
    pub icon: Option<String>,
    pub delay_seconds: Option<u32>,
    pub hidden: Option<bool>,
    pub terminal: Option<bool>,
    pub only_show_in: Option<Vec<String>>,
    pub not_show_in: Option<Vec<String>>,
}
