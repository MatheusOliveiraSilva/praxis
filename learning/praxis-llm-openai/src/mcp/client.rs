// MCP Client (JSON-RPC over stdio)
// Simplified implementation for stdio-based MCP servers

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::io::{BufRead, BufReader, Write};

/// MCP Tool schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP Client (connects via stdio)
pub struct MCPClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: u64,
}

impl MCPClient {
    /// Start MCP server process and connect
    pub fn connect(command: &str, args: &[&str]) -> Result<Self> {
        let mut process = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to spawn MCP server")?;
        
        let stdin = process.stdin.take().context("Failed to get stdin")?;
        let stdout = BufReader::new(process.stdout.take().context("Failed to get stdout")?);
        
        let mut client = Self {
            process,
            stdin,
            stdout,
            request_id: 1,
        };
        
        // Initialize connection
        client.initialize()?;
        
        Ok(client)
    }
    
    /// Initialize MCP connection
    fn initialize(&mut self) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "praxis-llm",
                    "version": "0.1.0"
                }
            }
        });
        
        self.send_request(request)?;
        let _response = self.read_response()?;
        
        // Send initialized notification
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        
        self.send_notification(notification)?;
        
        Ok(())
    }
    
    /// List available tools
    pub fn list_tools(&mut self) -> Result<Vec<MCPTool>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "tools/list",
            "params": {}
        });
        
        self.send_request(request)?;
        let response = self.read_response()?;
        
        let tools = response["result"]["tools"]
            .as_array()
            .context("Invalid tools response")?
            .iter()
            .map(|tool| serde_json::from_value(tool.clone()))
            .collect::<Result<Vec<MCPTool>, _>>()
            .context("Failed to parse tools")?;
        
        Ok(tools)
    }
    
    /// Call a tool
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });
        
        self.send_request(request)?;
        let response = self.read_response()?;
        
        if let Some(error) = response.get("error") {
            anyhow::bail!("MCP tool call error: {}", error);
        }
        
        let result = response["result"]["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .context("Invalid tool result")?
            .clone();
        
        Ok(result)
    }
    
    fn send_request(&mut self, request: Value) -> Result<()> {
        let mut json_str = serde_json::to_string(&request)?;
        json_str.push('\n');
        self.stdin.write_all(json_str.as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }
    
    fn send_notification(&mut self, notification: Value) -> Result<()> {
        let mut json_str = serde_json::to_string(&notification)?;
        json_str.push('\n');
        self.stdin.write_all(json_str.as_bytes())?;
        self.stdin.flush()?;
        Ok(())
    }
    
    fn read_response(&mut self) -> Result<Value> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        
        if line.trim().is_empty() {
            anyhow::bail!("Empty response from MCP server");
        }
        
        let response: Value = serde_json::from_str(&line)
            .context("Failed to parse JSON response")?;
        
        Ok(response)
    }
    
    fn next_id(&mut self) -> u64 {
        let id = self.request_id;
        self.request_id += 1;
        id
    }
}

impl Drop for MCPClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}
