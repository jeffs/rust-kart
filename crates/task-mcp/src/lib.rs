//! MCP server for managing background tasks.
//!
//! Provides tools to start, stop, list, and inspect background processes.

#![forbid(unsafe_code)]

mod error;
mod process;
mod state;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, ListToolsResult, PaginatedRequestParam,
    ServerInfo,
};
use rmcp::schemars::JsonSchema;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{tool, tool_router, ErrorData as McpError, ServerHandler};
use serde::{Deserialize, Serialize};

pub use error::{Error, Result};
pub use state::{TaskInfo, TaskManager};

// === Tool argument schemas ===

/// Arguments for the `task_ensure` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskEnsureArgs {
    /// Unique name for the task.
    #[schemars(description = "Unique name for the task")]
    pub name: String,
    /// Shell command to execute.
    #[schemars(description = "Shell command to execute")]
    pub command: String,
    /// Working directory (optional).
    #[schemars(description = "Working directory (optional)")]
    pub cwd: Option<String>,
}

/// Arguments for the `task_stop` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskStopArgs {
    /// Name of the task to stop.
    #[schemars(description = "Name of the task to stop")]
    pub name: String,
}

/// Arguments for the `task_logs` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct TaskLogsArgs {
    /// Name of the task.
    #[schemars(description = "Name of the task")]
    pub name: String,
    /// Number of lines to return (default: 50).
    #[schemars(description = "Number of lines to return (default: 50)")]
    pub tail: Option<usize>,
}

// === Tool response schemas ===

/// Status information for a task.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskStatus {
    /// Task name.
    pub name: String,
    /// Process ID.
    pub pid: u32,
    /// The command being run.
    pub command: String,
    /// Working directory.
    pub cwd: Option<String>,
    /// Whether the process is still alive.
    pub alive: bool,
    /// Seconds since the task was started.
    pub uptime_secs: u64,
}

/// Result of `task_ensure`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskEnsureResult {
    /// `"started"` or `"already_running"`.
    pub status: String,
    /// Task status information.
    pub task: TaskStatus,
}

/// Result of `task_stop`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskStopResult {
    /// Always "stopped".
    pub status: String,
    /// Name of the stopped task.
    pub name: String,
}

/// Result of `task_list`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskListResult {
    /// List of all tracked tasks.
    pub tasks: Vec<TaskStatus>,
}

/// Result of `task_logs`.
#[derive(Debug, Serialize, JsonSchema)]
pub struct TaskLogsResult {
    /// Task name.
    pub name: String,
    /// Recent stdout output.
    pub stdout: String,
    /// Recent stderr output.
    pub stderr: String,
}

// === MCP Server ===

/// Convert `TaskInfo` to `TaskStatus` for API responses.
fn task_to_status(info: &TaskInfo) -> TaskStatus {
    TaskStatus {
        name: info.name.clone(),
        pid: info.pid,
        command: info.command.clone(),
        cwd: info.cwd.as_ref().map(|p| p.display().to_string()),
        alive: process::is_alive(info.pid),
        uptime_secs: info.started_at.elapsed().as_secs(),
    }
}

/// MCP server for managing background tasks.
#[derive(Clone)]
pub struct TaskMcpServer {
    manager: TaskManager,
    tool_router: ToolRouter<Self>,
}

impl TaskMcpServer {
    /// Create a new task MCP server.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: TaskManager::new(),
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for TaskMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl TaskMcpServer {
    /// Ensure a background task is running.
    ///
    /// Idempotent: succeeds whether the task was started fresh or was already running.
    /// If the task exists but the process is dead, it will be restarted.
    #[tool(description = "Ensure a background task is running. Idempotent: succeeds whether task was started fresh or was already running.")]
    async fn task_ensure(
        &self,
        Parameters(args): Parameters<TaskEnsureArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let TaskEnsureArgs { name, command, cwd } = args;

        // Check if task already exists and is alive
        if let Some(existing) = self.manager.get(&name).await {
            if process::is_alive(existing.pid) {
                let result = TaskEnsureResult {
                    status: "already_running".to_string(),
                    task: task_to_status(&existing),
                };
                let json = serde_json::to_string_pretty(&result)
                    .map_err(|e| McpError::internal_error(e.to_string(), None))?;
                return Ok(CallToolResult::success(vec![Content::text(json)]));
            }
            // Task exists but process is dead - clean up and restart
            if let Some(old) = self.manager.remove(&name).await {
                process::cleanup_logs(&old.stdout_path, &old.stderr_path).await;
            }
        }

        // Spawn new task
        let cwd_path = cwd.as_ref().map(std::path::Path::new);
        let info = process::spawn_task(&name, &command, cwd_path)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        let status = task_to_status(&info);
        self.manager.insert(info).await;

        let result = TaskEnsureResult {
            status: "started".to_string(),
            task: status,
        };
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Stop a background task and clean up its log files.
    #[tool(description = "Stop a background task and clean up its log files.")]
    async fn task_stop(
        &self,
        Parameters(args): Parameters<TaskStopArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let TaskStopArgs { name } = args;

        let info = self.manager.remove(&name).await.ok_or_else(|| {
            McpError::invalid_params(format!("task not found: {name}"), None)
        })?;

        // Terminate process if alive
        if process::is_alive(info.pid) {
            let _ = process::terminate(info.pid);
        }

        // Clean up log files
        process::cleanup_logs(&info.stdout_path, &info.stderr_path).await;

        let result = TaskStopResult {
            status: "stopped".to_string(),
            name,
        };
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List all background tasks with their current status.
    #[tool(description = "List all background tasks with their current status.")]
    async fn task_list(&self) -> std::result::Result<CallToolResult, McpError> {
        let tasks = self.manager.list().await;
        let statuses: Vec<TaskStatus> = tasks.iter().map(task_to_status).collect();

        let result = TaskListResult { tasks: statuses };
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get the stdout and stderr logs from a background task.
    #[tool(description = "Get the stdout and stderr logs from a background task.")]
    async fn task_logs(
        &self,
        Parameters(args): Parameters<TaskLogsArgs>,
    ) -> std::result::Result<CallToolResult, McpError> {
        let TaskLogsArgs { name, tail } = args;

        let info = self.manager.get(&name).await.ok_or_else(|| {
            McpError::invalid_params(format!("task not found: {name}"), None)
        })?;

        let tail = tail.unwrap_or(50);

        let stdout = process::read_log_tail(&info.stdout_path, tail)
            .await
            .unwrap_or_default();
        let stderr = process::read_log_tail(&info.stderr_path, tail)
            .await
            .unwrap_or_default();

        let result = TaskLogsResult {
            name,
            stdout,
            stderr,
        };
        let json = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl ServerHandler for TaskMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Background task manager. Use task_ensure to start tasks, \
                 task_stop to terminate them, task_list to see all tasks, \
                 and task_logs to view output."
                    .to_string(),
            ),
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = std::result::Result<CallToolResult, McpError>> + Send + '_
    {
        let tool_context = ToolCallContext::new(self, request, context);
        async move { self.tool_router.call(tool_context).await }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = std::result::Result<ListToolsResult, McpError>> + Send + '_
    {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
            meta: None,
        }))
    }
}
