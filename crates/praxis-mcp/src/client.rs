use anyhow::Result;
use rmcp::transport::{TokioChildProcess, ConfigureCommandExt};
use rmcp::ServiceExt;
use serde_json::Value;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

type ServerSession = Arc<Mutex<Box<dyn std::any::Any + Send + Sync>>>;

/// MCP Client wrapper that manages connection to MCP servers
/// 
/// Supports stdio transport for connecting to local MCP servers  
pub struct MCPClient {
    server_name: String,
    server: ServerSession,
}

impl MCPClient {
    /// Create a new MCP client via stdio (spawns local process)
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// let client = MCPClient::new_stdio(
    ///     "weather",
    ///     "python3",
    ///     vec!["weather.py"]
    /// ).await?;
    /// ```
    pub async fn new_stdio(
        server_name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Result<Self> {
        let server_name = server_name.into();
        let command = command.into();
        let args: Vec<String> = args.into_iter().map(|a| a.into()).collect();
        
        // Build and configure command
        let cmd = Command::new(&command).configure(|c| {
            for arg in &args {
                c.arg(arg);
            }
            c.stdin(Stdio::piped());
            c.stdout(Stdio::piped());
            c.stderr(Stdio::inherit());
        });

        // Spawn MCP server process and connect
        let transport = TokioChildProcess::new(cmd)?;
        let server = ().serve(transport).await?;

        Ok(Self {
            server_name,
            server: Arc::new(Mutex::new(Box::new(server))),
        })
    }

    /// List all available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        // For now return mock data - full rmcp integration coming soon
        // TODO: Implement proper rmcp session management
        let tools = vec![
            ToolInfo {
                name: "get_alerts".to_string(),
                description: Some("Get weather alerts for a US state".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "state": {
                            "type": "string",
                            "description": "Two-letter US state code"
                        }
                    }
                }),
            },
            ToolInfo {
                name: "get_forecast".to_string(),
                description: Some("Get weather forecast for a location".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "latitude": { "type": "number" },
                        "longitude": { "type": "number" }
                    }
                }),
            },
        ];
        
        Ok(tools)
    }
    

    /// Call a tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<ToolResponse>> {
        // For now return mock responses
        // TODO: Implement proper rmcp tool calling
        let result = format!(
            "Mock MCP response for {}: arguments = {}",
            name,
            serde_json::to_string_pretty(&arguments)?
        );
        
        Ok(vec![ToolResponse::Text { text: result }])
    }

    /// Get server name
    pub fn name(&self) -> &str {
        &self.server_name
    }
}

/// Tool information from MCP server
#[derive(Debug, Clone)]
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

/// Response from tool execution
#[derive(Debug, Clone)]
pub enum ToolResponse {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, text: Option<String>, mime_type: Option<String> },
}

impl ToolResponse {
    /// Convert response to string representation
    pub fn to_string(&self) -> String {
        match self {
            Self::Text { text } => text.clone(),
            Self::Image { mime_type, .. } => format!("[Image: {}]", mime_type),
            Self::Resource { uri, text, .. } => {
                if let Some(text) = text {
                    format!("{}\n{}", uri, text)
                } else {
                    uri.clone()
                }
            }
        }
    }

    /// Convert all responses to a single string
    pub fn join_responses(responses: &[ToolResponse]) -> String {
        responses
            .iter()
            .map(|r| r.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

