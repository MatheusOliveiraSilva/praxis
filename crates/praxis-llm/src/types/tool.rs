use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool/Function definition (sent to OpenAI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String, // Always "function" for now
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// JSON Schema for parameters
    pub parameters: Value,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

impl Tool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: name.into(),
                description: Some(description.into()),
                parameters,
                strict: None,
            },
        }
    }
}

/// Tool call made by the LLM (in assistant message)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    
    #[serde(rename = "type")]
    pub tool_type: String, // "function"
    
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

impl ToolCall {
    /// Parse arguments as JSON
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }
    
    /// Get arguments as Value
    pub fn arguments_value(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.function.arguments)
    }
}

/// Tool choice parameter (how aggressive to use tools)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// "auto" - let model decide
    Auto(String),
    
    /// "none" - don't use tools
    None(String),
    
    /// "required" - must use at least one tool
    Required(String),
    
    /// Force specific tool
    Specific {
        #[serde(rename = "type")]
        tool_type: String,
        function: ToolChoiceFunction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChoiceFunction {
    pub name: String,
}

impl ToolChoice {
    pub fn auto() -> Self {
        Self::Auto("auto".to_string())
    }
    
    pub fn none() -> Self {
        Self::None("none".to_string())
    }
    
    pub fn required() -> Self {
        Self::Required("required".to_string())
    }
    
    pub fn force(tool_name: impl Into<String>) -> Self {
        Self::Specific {
            tool_type: "function".to_string(),
            function: ToolChoiceFunction {
                name: tool_name.into(),
            },
        }
    }
}
