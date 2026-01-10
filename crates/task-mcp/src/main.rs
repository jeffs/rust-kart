//! MCP server for managing background tasks.

use rmcp::transport::stdio;
use rmcp::ServiceExt;
use task_mcp::TaskMcpServer;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let server = TaskMcpServer::new().serve(stdio()).await?;
    server.waiting().await?;
    Ok(())
}
