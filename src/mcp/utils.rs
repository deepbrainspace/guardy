//! MCP utility functions
//!
//! This module provides utility functions for the MCP server.

use crate::mcp::types::*;
use serde_json::{Value, json};

/// Create a successful MCP response
/// TODO: Remove #[allow(dead_code)] when MCP server is implemented in Phase 1.6
#[allow(dead_code)]
pub fn create_success_response(id: Value, result: Value) -> McpResponseMessage {
    McpResponseMessage {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

/// Create an error MCP response
#[allow(dead_code)]
pub fn create_error_response(id: Value, code: i32, message: String) -> McpResponseMessage {
    McpResponseMessage {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(McpErrorResponse {
            code,
            message,
            data: None,
        }),
    }
}

/// Create text content
#[allow(dead_code)]
pub fn create_text_content(text: String) -> McpContent {
    McpContent::Text { text }
}

/// Create image content
#[allow(dead_code)]
pub fn create_image_content(data: String, mime_type: String) -> McpContent {
    McpContent::Image { data, mime_type }
}

/// Create resource content
#[allow(dead_code)]
pub fn create_resource_content(uri: String, text: Option<String>) -> McpContent {
    McpContent::Resource {
        resource: McpResourceReference { uri, text },
    }
}

/// Validate JSON-RPC request
#[allow(dead_code)]
pub fn validate_jsonrpc_request(request: &Value) -> Result<(), String> {
    if !request.is_object() {
        return Err("Request must be an object".to_string());
    }

    let obj = request.as_object().unwrap();

    // Check jsonrpc version
    if let Some(version) = obj.get("jsonrpc") {
        if version != "2.0" {
            return Err("Invalid JSON-RPC version".to_string());
        }
    } else {
        return Err("Missing jsonrpc field".to_string());
    }

    // Check method
    if let Some(method) = obj.get("method") {
        if !method.is_string() {
            return Err("Method must be a string".to_string());
        }
    } else {
        return Err("Missing method field".to_string());
    }

    // Check id (required for requests)
    if obj.get("id").is_none() {
        return Err("Missing id field".to_string());
    }

    Ok(())
}

/// Extract parameters from request
#[allow(dead_code)]
pub fn extract_params(request: &Value) -> Option<Value> {
    request.as_object()?.get("params").cloned()
}

/// Create tool list response
#[allow(dead_code)]
pub fn create_tool_list_response(tools: Vec<McpToolDefinition>) -> Value {
    json!({
        "tools": tools
    })
}

/// Create initialize response
#[allow(dead_code)]
pub fn create_initialize_response() -> Value {
    json!({
        "protocolVersion": MCP_VERSION,
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "guardy-mcp",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}
