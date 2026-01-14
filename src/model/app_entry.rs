use crate::desktop_entry::DesktopEntry;

#[derive(Debug, Clone)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub comment: Option<String>,
    pub keywords: Vec<String>,
}

impl Application {
    pub fn from_desktop_entry(id: String, entry: &DesktopEntry) -> Self {
        Self {
            id,
            name: entry.name.clone(),
            exec: entry.exec.clone(),
            icon: entry.icon.clone(),
            comment: entry.comment.clone(),
            keywords: entry.keywords.clone(),
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.exec.to_lowercase().contains(&query)
            || self
                .comment
                .as_ref()
                .map(|c| c.to_lowercase().contains(&query))
                .unwrap_or(false)
            || self
                .keywords
                .iter()
                .any(|k| k.to_lowercase().contains(&query))
    }
}
