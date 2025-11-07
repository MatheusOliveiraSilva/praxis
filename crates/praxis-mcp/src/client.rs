use anyhow::Result;
use rmcp::{ServiceExt, service::RoleClient};
use rmcp::transport::streamable_http_client::StreamableHttpClientWorker;
use serde_json::Value;

/// MCP Client wrapper that manages connection to MCP servers
/// 
/// Maintains an active connection to an MCP server and provides methods
/// to list tools and execute them. The connection is kept alive for the
/// lifetime of this client.
/// 
/// # Examples
/// 
/// ```no_run
/// use praxis_mcp::MCPClient;
/// 
/// # async fn example() -> anyhow::Result<()> {
/// // Connect to MCP server
/// let client = MCPClient::new_http(
///     "weather",
///     "http://localhost:8000"
/// ).await?;
/// 
/// // List available tools
/// let tools = client.list_tools().await?;
/// 
/// // Call a tool
/// let result = client.call_tool("get_weather", serde_json::json!({
///     "location": "San Francisco"
/// })).await?;
/// # Ok(())
/// # }
/// ```
pub struct MCPClient {
    server_name: String,
    /// Keep the running service alive (connection stays open)
    _running_service: rmcp::service::RunningService<RoleClient, ()>,
    /// Peer for making MCP calls
    peer: rmcp::service::Peer<RoleClient>,
}

impl MCPClient {
    /// Create a new MCP client via HTTP (streamable-http transport)
    /// 
    /// Connects to an MCP server running with streamable-http transport.
    /// The connection is established during this call and kept alive for
    /// the lifetime of the MCPClient.
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use praxis_mcp::MCPClient;
    /// 
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = MCPClient::new_http(
    ///     "weather",
    ///     "http://localhost:8000"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_http(
        server_name: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<Self> {
        let server_name = server_name.into();
        let url = url.into();
        
        // Create streamable HTTP worker as transport using reqwest::Client
        let worker = StreamableHttpClientWorker::<reqwest::Client>::new_simple(url.clone());
        
        // Connect and perform MCP handshake (initialize/initialized)
        // The worker itself implements the Worker trait which can be used as transport
        let running_service = ().serve(worker).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to MCP server at {}: {}", url, e))?;
        
        // Get peer for making calls (clone to own it)
        let peer = running_service.peer().clone();
        
        Ok(Self {
            server_name,
            _running_service: running_service,
            peer,
        })
    }

    /// List all available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        use rmcp::model::PaginatedRequestParam;
        
        // Call MCP list_tools
        let result = self.peer.list_tools(Some(PaginatedRequestParam { cursor: None })).await
            .map_err(|e| anyhow::anyhow!("Failed to list tools: {}", e))?;
        
        // Convert rmcp::Tool to our ToolInfo
        Ok(result.tools.into_iter().map(|tool| ToolInfo {
            name: tool.name.to_string(),
            description: tool.description.map(|d| d.to_string()),
            input_schema: serde_json::Value::Object((*tool.input_schema).clone()),
        }).collect())
    }

    /// Call a tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<ToolResponse>> {
        use rmcp::model::CallToolRequestParam;
        
        // Convert Value to Option<Map> for MCP
        let arguments_map = match arguments {
            Value::Object(map) => Some(map),
            _ => None,
        };
        
        let param = CallToolRequestParam {
            name: name.to_string().into(),
            arguments: arguments_map,
        };
        
        let result = self.peer.call_tool(param).await
            .map_err(|e| anyhow::anyhow!("Failed to call tool '{}': {}", name, e))?;
        
        // Convert MCP content to ToolResponse
        Ok(result.content.into_iter().map(|content| {
            // For now, serialize all content as text
            // In future, handle different content types properly
            ToolResponse::Text { 
                text: serde_json::to_string(&content).unwrap_or_else(|_| "".to_string())
            }
        }).collect())
    }
    
    /// Get tools in format suitable for LLM (praxis_llm::Tool)
    /// 
    /// This fetches tools from the MCP server and converts them to the format
    /// expected by the LLM client.
    pub async fn get_llm_tools(&self) -> Result<Vec<praxis_llm::Tool>> {
        let tools = self.list_tools().await?;
        
        Ok(tools.into_iter().map(|t| {
            praxis_llm::Tool::new(
                t.name,
                t.description.unwrap_or_default(),
                t.input_schema,
            )
        }).collect())
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

