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
    #[serde(default)]
    pub observability: ObservabilityConfig,
    
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

impl From<LlmConfig> for praxis::LLMConfig {
    fn from(config: LlmConfig) -> Self {
        Self {
            model: config.model,
            temperature: None, 
            max_tokens: None,
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

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub langfuse: LangfuseConfig,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "langfuse".to_string(),
            langfuse: LangfuseConfig::default(),
        }
    }
}

fn default_provider() -> String {
    "langfuse".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct LangfuseConfig {
    #[serde(default)]
    pub public_key: String,
    #[serde(default)]
    pub secret_key: String,
    #[serde(default = "default_langfuse_host")]
    pub host: String,
}

impl Default for LangfuseConfig {
    fn default() -> Self {
        Self {
            public_key: String::new(),
            secret_key: String::new(),
            host: "https://cloud.langfuse.com".to_string(),
        }
    }
}

fn default_langfuse_host() -> String {
    "https://cloud.langfuse.com".to_string()
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
            )
            .add_source(
                Environment::default()
                    .prefix("OBSERVABILITY")
                    .separator("_")
                    .try_parsing(true)
            )
            .add_source(
                Environment::default()
                    .prefix("LANGFUSE")
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
        
        if let Ok(enabled) = std::env::var("OBSERVABILITY_ENABLED") {
            cfg.observability.enabled = enabled.to_lowercase() == "true" || enabled == "1";
        }
        if let Ok(host) = std::env::var("LANGFUSE_HOST") {
            cfg.observability.langfuse.host = host;
        }
        if let Ok(public_key) = std::env::var("LANGFUSE_PUBLIC_KEY") {
            cfg.observability.langfuse.public_key = public_key;
        }
        if let Ok(secret_key) = std::env::var("LANGFUSE_SECRET_KEY") {
            cfg.observability.langfuse.secret_key = secret_key;
        }
        
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

