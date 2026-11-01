use std::sync::Arc;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// What the progress belongs to (an instance id, usually).
    pub task: String,
    /// Short machine-readable stage: `version`, `libraries`, `assets`, `java`, `mods`, `launch`.
    pub stage: String,
    pub done: u64,
    pub total: u64,
    /// Human-readable line for the UI.
    pub message: String,
}

pub type ProgressFn = Arc<dyn Fn(Progress) + Send + Sync>;

#[derive(Clone)]
pub struct Reporter {
    pub task: String,
    f: ProgressFn,
}

impl Reporter {
    pub fn new(task: impl Into<String>, f: ProgressFn) -> Self {
        Self { task: task.into(), f }
    }

    pub fn silent(task: impl Into<String>) -> Self {
        Self::new(task, Arc::new(|_| {}))
    }

    pub fn report(&self, stage: &str, done: u64, total: u64, message: impl Into<String>) {
        (self.f)(Progress { task: self.task.clone(), stage: stage.into(), done, total, message: message.into() });
    }
}
