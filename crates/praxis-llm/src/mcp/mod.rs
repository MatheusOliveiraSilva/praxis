// MCP (Model Context Protocol) integration

pub mod client;
pub mod adapter;

pub use client::{MCPClient, MCPTool};
pub use adapter::{mcp_to_openai_tool, mcp_tools_to_openai};
