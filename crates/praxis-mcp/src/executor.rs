use crate::client::{MCPClient, ToolResponse};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool executor that delegates to MCP servers
pub struct MCPToolExecutor {
    clients: Arc<RwLock<HashMap<String, Arc<MCPClient>>>>,
}

impl MCPToolExecutor {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an MCP server
    pub async fn add_server(&self, client: MCPClient) -> Result<()> {
        let name = client.name().to_string();
        let mut clients = self.clients.write().await;
        clients.insert(name, Arc::new(client));
        Ok(())
    }

    /// List all available tools from all connected MCP servers
    pub async fn list_all_tools(&self) -> Result<Vec<(String, Vec<crate::client::ToolInfo>)>> {
        let clients = self.clients.read().await;
        let mut all_tools = Vec::new();

        for (server_name, client) in clients.iter() {
            let tools = client.list_tools().await?;
            all_tools.push((server_name.clone(), tools));
        }

        Ok(all_tools)
    }

    /// Execute a tool by finding the right MCP server
    #[allow(dead_code)]
    async fn execute_tool(&self, tool_name: &str, arguments: &str) -> Result<String> {
        let clients = self.clients.read().await;

        // Try to find the tool in any of the connected servers
        for client in clients.values() {
            let tools = client.list_tools().await?;
            
            if tools.iter().any(|t| t.name == tool_name) {
                // Found the tool! Call it
                let args: serde_json::Value = serde_json::from_str(arguments)?;
                let responses = client.call_tool(tool_name, args).await?;
                return Ok(ToolResponse::join_responses(&responses));
            }
        }

        Err(anyhow::anyhow!("Tool '{}' not found in any connected MCP server", tool_name))
    }
}

// Note: We're intentionally NOT implementing the ToolExecutor trait here
// because we want to deprecate the mock-based approach.
// Instead, MCPToolExecutor provides its own interface that's MCP-native.

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = MCPToolExecutor::new();
        assert!(executor.list_all_tools().await.unwrap().is_empty());
    }
}

