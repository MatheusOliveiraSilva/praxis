use anyhow::Result;
use serde_json::Value;

/// Transport type for MCP connection
#[derive(Debug, Clone)]
pub enum MCPTransport {
    /// Connect via stdio to a local process (npx, python, etc.)
    Stdio {
        command: String,
        args: Vec<String>,
    },
    /// Connect via HTTP to a remote MCP server
    Http {
        url: String,
        headers: Vec<(String, String)>,
    },
}

/// MCP Client wrapper that manages connection to MCP servers
/// 
/// Supports two transport modes:
/// - **Stdio**: For local development and spawning MCP server processes
/// - **HTTP**: For production deployments with remote MCP servers
/// 
/// # Examples
/// 
/// ## Stdio Transport (Development)
/// ```no_run
/// let client = MCPClient::new_stdio(
///     "weather",
///     "python3",
///     vec!["weather_server.py"]
/// ).await?;
/// ```
/// 
/// ## HTTP Transport (Production)
/// ```no_run
/// let client = MCPClient::new_http(
///     "weather",
///     "https://mcp.example.com/weather"
/// ).await?;
/// ```
pub struct MCPClient {
    server_name: String,
    transport: MCPTransport,
    // TODO: Add actual rmcp client connection
    // When implemented, this will hold the active rmcp connection
}

impl MCPClient {
    /// Create a new MCP client via **stdio** (spawns local process)
    /// 
    /// Best for: Development, testing, local tools
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// // Python MCP server
    /// let client = MCPClient::new_stdio(
    ///     "weather",
    ///     "python3",
    ///     vec!["weather_server.py"]
    /// ).await?;
    /// ```
    pub async fn new_stdio(
        server_name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Result<Self> {
        let server_name = server_name.into();
        let transport = MCPTransport::Stdio {
            command: command.into(),
            args: args.into_iter().map(|a| a.into()).collect(),
        };
        
        // TODO: Spawn process and connect via rmcp
        // use rmcp::transport::TokioChildProcess;
        // use rmcp::ServiceExt;
        // 
        // let mut cmd = Command::new(&transport.command);
        // cmd.configure(|c| {
        //     for arg in &transport.args {
        //         c.arg(arg);
        //     }
        //     c.stdin(Stdio::piped())
        //         .stdout(Stdio::piped())
        //         .stderr(Stdio::inherit())
        // });
        // let process = TokioChildProcess::new(cmd)?;
        // let client = ().serve(process).await?;
        
        Ok(Self {
            server_name,
            transport,
        })
    }

    /// Create a new MCP client via **HTTP** (connects to remote server)
    /// 
    /// Best for: Production, distributed systems, remote tools
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// // Basic HTTP connection
    /// let client = MCPClient::new_http(
    ///     "weather",
    ///     "https://mcp.example.com/weather"
    /// ).await?;
    /// ```
    pub async fn new_http(
        server_name: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<Self> {
        let server_name = server_name.into();
        let transport = MCPTransport::Http {
            url: url.into(),
            headers: vec![],
        };
        
        // TODO: Connect via rmcp HTTP transport
        // use rmcp::transport::StreamableHttpClientTransport;
        // use rmcp::ServiceExt;
        // 
        // let http_transport = StreamableHttpClientTransport::new(&transport.url)?;
        // let client = ().serve(http_transport).await?;
        
        Ok(Self {
            server_name,
            transport,
        })
    }

    /// Add HTTP header (only for HTTP transport)
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let MCPTransport::Http { headers, .. } = &mut self.transport {
            headers.push((key.into(), value.into()));
        }
        self
    }

    /// Legacy method for backwards compatibility
    /// 
    /// Deprecated: Use `new_stdio` or `new_http` instead
    #[deprecated(since = "0.2.0", note = "Use new_stdio() or new_http() instead")]
    pub async fn new(
        server_name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<&str>,
    ) -> Result<Self> {
        Self::new_stdio(server_name, command, args).await
    }

    /// List all available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        // TODO: Implement using rmcp.list_tools()
        Ok(vec![
            ToolInfo {
                name: format!("{}_tool_1", self.server_name),
                description: Some(format!("Example tool from {}", self.server_name)),
                input_schema: Value::Object(serde_json::Map::new()),
            },
        ])
    }

    /// Call a tool on the MCP server
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<ToolResponse>> {
        // TODO: Implement using rmcp.call_tool()
        Ok(vec![ToolResponse::Text {
            text: format!("Mock response from {}: {} with {:?}", self.server_name, name, arguments),
        }])
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

