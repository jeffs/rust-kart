//! Task state management.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Mutex;

/// Information about a managed task.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Unique name for this task.
    pub name: String,
    /// Process ID of the running task.
    pub pid: u32,
    /// The command that was executed.
    pub command: String,
    /// Working directory the command runs in.
    pub cwd: Option<PathBuf>,
    /// Path to the stdout log file.
    pub stdout_path: PathBuf,
    /// Path to the stderr log file.
    pub stderr_path: PathBuf,
    /// When the task was started.
    pub started_at: Instant,
}

/// Manages the collection of tracked tasks.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<String, TaskInfo>>>,
}

impl TaskManager {
    /// Create a new empty task manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a task by name.
    pub async fn get(&self, name: &str) -> Option<TaskInfo> {
        let tasks = self.tasks.lock().await;
        tasks.get(name).cloned()
    }

    /// Insert or update a task.
    pub async fn insert(&self, info: TaskInfo) {
        let mut tasks = self.tasks.lock().await;
        tasks.insert(info.name.clone(), info);
    }

    /// Remove a task by name, returning it if it existed.
    pub async fn remove(&self, name: &str) -> Option<TaskInfo> {
        let mut tasks = self.tasks.lock().await;
        tasks.remove(name)
    }

    /// List all tracked tasks.
    pub async fn list(&self) -> Vec<TaskInfo> {
        let tasks = self.tasks.lock().await;
        tasks.values().cloned().collect()
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}
