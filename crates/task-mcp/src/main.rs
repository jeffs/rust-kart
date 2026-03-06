//! MCP server for managing background tasks.

use rmcp::ServiceExt;
use rmcp::transport::stdio;
use task_mcp::TaskMcpServer;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = TaskMcpServer::new().serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
