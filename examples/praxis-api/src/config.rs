use config::{Config as ConfigLoader, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub cors: CorsConfig,
    pub mongodb: MongoDbConfig,
    pub llm: LlmConfig,
    pub mcp: McpConfig,
    pub logging: LoggingConfig,
    
    // Secrets (from ENV only)
    #[serde(default)]
    pub mongodb_uri: String,
    #[serde(default)]
    pub openai_api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub workers: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MongoDbConfig {
    pub database: String,
    pub pool_size: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    pub model: String,
    pub temperature: f32,
    /// Max tokens for context window management (NOT sent to OpenAI)
    pub max_tokens: usize,
}

impl From<LlmConfig> for praxis_types::LLMConfig {
    fn from(config: LlmConfig) -> Self {
        Self {
            model: config.model,
            temperature: None,  // Never send temperature to OpenAI - let it decide
            max_tokens: None,  // Never send max_tokens to OpenAI - let it decide
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    pub servers: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    /// Load configuration from TOML files and environment variables
    /// 
    /// Hierarchy (weakest to strongest):
    /// 1. config/default.toml
    /// 2. config/{ENV}.toml (if ENV is set)
    /// 3. Environment variables (with SERVER_, MONGODB_, LLM_, etc. prefixes)
    pub fn load() -> Result<Self, ConfigError> {
        let env = std::env::var("ENV").unwrap_or_else(|_| "dev".to_string());
        
        let builder = ConfigLoader::builder()
            // 1. Load default config
            .add_source(File::with_name("config/default").required(false))
            // 2. Load environment-specific config
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // 3. Environment variables override everything
            .add_source(
                Environment::default()
                    .prefix("SERVER")
                    .separator("_")
                    .try_parsing(true)
            )
            .add_source(
                Environment::default()
                    .prefix("MONGODB")
                    .separator("_")
                    .try_parsing(true)
            )
            .add_source(
                Environment::default()
                    .prefix("LLM")
                    .separator("_")
                    .try_parsing(true)
            )
            .add_source(
                Environment::default()
                    .prefix("LOG")
                    .separator("_")
                    .try_parsing(true)
            )
            .add_source(
                Environment::default()
                    .prefix("MCP")
                    .separator("_")
                    .try_parsing(true)
            );
        
        let config = builder.build()?;
        
        let mut cfg: Config = config.try_deserialize()?;
        
        // Load secrets from ENV (not in TOML)
        cfg.mongodb_uri = std::env::var("MONGODB_URI")
            .map_err(|_| ConfigError::Message("MONGODB_URI environment variable is required".to_string()))?;
        cfg.openai_api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| ConfigError::Message("OPENAI_API_KEY environment variable is required".to_string()))?;
        
        Ok(cfg)
    }
    
    /// Load config from a specific path (useful for testing)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let builder = ConfigLoader::builder()
            .add_source(File::from(path.as_ref()));
        
        let config = builder.build()?;
        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_structure() {
        // Test that config structure is valid
        let toml = r#"
            [server]
            host = "127.0.0.1"
            port = 3000
            workers = 4
            
            [cors]
            enabled = true
            origins = ["http://localhost:3000"]
            
            [mongodb]
            database = "test"
            pool_size = 5
            timeout_ms = 3000
            
            [llm]
            model = "gpt-4"
            temperature = 0.5
            max_tokens = 8000
            
            [mcp]
            servers = "http://localhost:8000/mcp"
            
            [logging]
            level = "debug"
            format = "json"
        "#;
        
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.mongodb.database, "test");
    }
}

