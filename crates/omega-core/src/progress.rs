use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub session: String,
    #[serde(default)]
    pub todos_total: u32,
    #[serde(default)]
    pub todos_completed: u32,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default)]
    pub current_task: Option<String>,
}

impl ProgressInfo {
    pub fn percentage(&self) -> f32 {
        if self.todos_total == 0 {
            return 0.0;
        }
        (self.todos_completed as f32 / self.todos_total as f32) * 100.0
    }

    pub fn bar(&self, width: usize) -> String {
        let pct = self.percentage() / 100.0;
        let filled = (pct * width as f32) as usize;
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    }

    pub fn read(state_dir: &Path, session: &str) -> Option<Self> {
        let path = state_dir.join(format!("{}.progress.json", session));
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn read_all(state_dir: &Path) -> Vec<Self> {
        let mut results = Vec::new();
        if let Ok(entries) = std::fs::read_dir(state_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".progress.json") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Ok(info) = serde_json::from_str::<ProgressInfo>(&content) {
                                    results.push(info);
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}
