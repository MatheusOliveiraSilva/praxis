// Adapter to convert MCP tools to OpenAI function format

use crate::mcp::client::MCPTool;
use crate::types::Tool;

/// Convert MCP tool to OpenAI function format
pub fn mcp_to_openai_tool(mcp_tool: &MCPTool) -> Tool {
    Tool {
        tool_type: "function".to_string(),
        function: crate::types::FunctionDefinition {
            name: mcp_tool.name.clone(),
            description: mcp_tool.description.clone(),
            parameters: mcp_tool.input_schema.clone(),
            strict: None,
        },
    }
}

/// Convert multiple MCP tools
pub fn mcp_tools_to_openai(mcp_tools: &[MCPTool]) -> Vec<Tool> {
    mcp_tools
        .iter()
        .map(mcp_to_openai_tool)
        .collect()
}
