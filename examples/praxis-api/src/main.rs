use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use praxis_api::{
    config::Config,
    middleware::logging,
    routes::{health, messages, threads},
    handlers::stream,
    state::AppState,
};
use praxis_llm::OpenAIClient;
use praxis_mcp::{MCPClient, MCPToolExecutor};
use praxis_persist::PersistClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Load configuration
    let config = Config::load()
        .map_err(|e| anyhow::anyhow!("Failed to load configuration: {}", e))?;
    
    // Initialize logging
    init_logging(&config);
    
    tracing::info!("Starting Praxis API server");
    tracing::info!("Config loaded: {}:{}", config.server.host, config.server.port);
    
    // Initialize LLM client
    tracing::info!("Initializing LLM client");
    let llm_client: Arc<dyn praxis_llm::LLMClient> = Arc::new(OpenAIClient::new(config.openai_api_key.clone())?);
    
    // Initialize MCP executor and connect to servers
    tracing::info!("Connecting to MCP servers");
    let mcp_executor = MCPToolExecutor::new();
    for (idx, url) in config.mcp.servers.split(',').enumerate() {
        let url = url.trim();
        if !url.is_empty() {
            match MCPClient::new_http(&format!("mcp-server-{}", idx), url).await {
                Ok(client) => {
                    mcp_executor.add_server(client).await?;
                    tracing::info!("Connected to MCP server: {}", url);
                }
                Err(e) => {
                    tracing::warn!("Failed to connect to MCP server {}: {}", url, e);
                }
            }
        }
    }
    
    // Initialize persistence client (MongoDB)
    tracing::info!("Connecting to MongoDB");
    let mongo_client = praxis_persist::MongoPersistenceClient::connect(
        &config.mongodb_uri,
        &config.mongodb.database,
    ).await?;
    let persist_client: Arc<dyn praxis_persist::PersistenceClient> = Arc::new(mongo_client);
    
    tracing::info!("MongoDB connected");
    
    // Create context strategy
    tracing::info!("Initializing context strategy");
    let context_strategy: Arc<dyn praxis_context::ContextStrategy> = Arc::new(
        praxis_context::DefaultContextStrategy::new(
            config.llm.max_tokens,
            llm_client.clone(),
        )
    );
    
    // Wrap mcp_executor in Arc for sharing
    let mcp_executor = Arc::new(mcp_executor);
    
    // Create graph with persistence
    tracing::info!("Initializing Graph orchestrator with persistence");
    let graph = praxis_graph::Graph::builder()
        .llm_client(llm_client.clone())
        .mcp_executor(Arc::clone(&mcp_executor))
        .with_persistence(persist_client.clone())
        .build()?;
    
    // Create application state
    let state = Arc::new(AppState::new(
        config.clone(),
        persist_client,
        context_strategy,
        llm_client,
        mcp_executor,
        graph,
    ));
    
    // Build router
    let app = build_router(state.clone());
    
    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server listening on {}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("API docs: http://{}/api/docs", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn build_router(state: Arc<AppState>) -> Router {
    // API routes
    let api_routes = Router::new()
        // Health
        .route("/health", get(health::health_check))
        // Threads
        .route("/threads", post(threads::create_thread))
        .route("/threads", get(threads::list_threads))
        .route("/threads/:thread_id", get(threads::get_thread))
        .route("/threads/:thread_id", delete(threads::delete_thread))
        // Messages
        .route("/threads/:thread_id/messages", get(messages::list_messages))
        .route("/threads/:thread_id/messages", post(stream::send_message_stream));
    
    // Build full router with middleware
    Router::new()
        .nest("/", api_routes)
        .layer(middleware::from_fn(logging::log_request))
        .layer(TimeoutLayer::new(std::time::Duration::from_secs(300)))
        .layer(CompressionLayer::new())
        .layer(build_cors_layer(&state.config))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    if config.cors.enabled {
        let cors = CorsLayer::new()
            .allow_methods([
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ])
            .allow_headers(Any);
        
        if config.cors.origins.iter().any(|o| o == "*") {
            cors.allow_origin(Any)
        } else {
            // Parse all origins and collect them
            let parsed_origins: Vec<axum::http::HeaderValue> = config.cors.origins
                .iter()
                .filter_map(|o| o.parse::<axum::http::HeaderValue>().ok())
                .collect();
            
            cors.allow_origin(parsed_origins)
        }
    } else {
        CorsLayer::permissive()
    }
}

fn init_logging(config: &Config) {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&config.logging.level))
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let registry = tracing_subscriber::registry().with(env_filter);
    
    match config.logging.format.as_str() {
        "json" => {
            registry
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        _ => {
            registry
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }
}

