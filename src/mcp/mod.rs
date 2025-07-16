//! Model Context Protocol (MCP) server implementation
//!
//! This module provides the built-in MCP server for AI integration.
//! The server exposes tools for project analysis, configuration generation,
//! and workflow management.

use anyhow::Result;
use axum::{
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

pub mod tools;
pub mod types;
pub mod utils;

/// MCP server state
/// TODO: Remove #[allow(dead_code)] when MCP server is implemented in Phase 1.6
#[allow(dead_code)]
#[derive(Clone)]
pub struct McpServer {
    /// Server configuration
    config: crate::config::McpConfig,
}

/// MCP JSON-RPC request
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// MCP JSON-RPC response
#[derive(Debug, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// MCP error response
#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

#[allow(dead_code)]
impl McpServer {
    /// Create a new MCP server
    pub fn new(config: crate::config::McpConfig) -> Self {
        Self { config }
    }

    /// Create the router for the MCP server
    pub fn router(&self) -> Router {
        Router::new()
            .route("/", get(health_check))
            .route("/mcp", post(handle_mcp_request))
            .with_state(self.clone())
    }

    /// Start the MCP server
    pub async fn start(&self) -> Result<()> {
        let app = self.router();
        let addr = format!("{}:{}", self.config.host, self.config.port);

        println!("🚀 Starting MCP server on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

/// Health check endpoint
#[allow(dead_code)]
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "guardy-mcp",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Handle MCP JSON-RPC requests
#[allow(dead_code)]
async fn handle_mcp_request(Json(request): Json<McpRequest>) -> impl IntoResponse {
    let response = match request.method.as_str() {
        "initialize" => handle_initialize(request.id, request.params).await,
        "tools/list" => handle_tools_list(request.id).await,
        "tools/call" => handle_tools_call(request.id, request.params).await,
        _ => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(McpError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        },
    };

    (StatusCode::OK, Json(response))
}

/// Handle initialize request
#[allow(dead_code)]
async fn handle_initialize(
    id: Option<serde_json::Value>,
    _params: Option<serde_json::Value>,
) -> McpResponse {
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "guardy-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
    }
}

/// Handle tools list request
#[allow(dead_code)]
async fn handle_tools_list(id: Option<serde_json::Value>) -> McpResponse {
    let tools = tools::get_available_tools();

    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "tools": tools
        })),
        error: None,
    }
}

/// Handle tools call request
#[allow(dead_code)]
async fn handle_tools_call(
    id: Option<serde_json::Value>,
    _params: Option<serde_json::Value>,
) -> McpResponse {
    // TODO: Implement tool execution
    McpResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::json!({
            "content": [
                {
                    "type": "text",
                    "text": "Tool execution not yet implemented"
                }
            ]
        })),
        error: None,
    }
}
