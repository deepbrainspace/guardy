//! MCP type definitions
//!
//! This module contains type definitions for the MCP protocol.

use serde::{Deserialize, Serialize};

/// MCP protocol version
/// TODO: Remove #[allow(dead_code)] when MCP server is implemented in Phase 1.6
#[allow(dead_code)]
pub const MCP_VERSION: &str = "2024-11-05";

/// MCP message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpMessage {
    #[serde(rename = "request")]
    Request(McpRequestMessage),
    #[serde(rename = "response")]
    Response(McpResponseMessage),
    #[serde(rename = "notification")]
    Notification(McpNotificationMessage),
}

/// MCP request message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequestMessage {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// MCP response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseMessage {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpErrorResponse>,
}

/// MCP notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNotificationMessage {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// MCP error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpErrorResponse {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// MCP capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

/// Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub version: String,
}

/// Client info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub name: String,
    pub arguments: Option<serde_json::Value>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub content: Vec<McpContent>,
    pub is_error: Option<bool>,
}

/// MCP content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: McpResourceReference },
}

/// Resource reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceReference {
    pub uri: String,
    pub text: Option<String>,
}
