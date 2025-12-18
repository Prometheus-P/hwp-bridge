// crates/hwp-mcp/src/main.rs

//! HWP MCP Server
//!
//! Model Context Protocol (MCP) server for HWP document processing.
//! Provides tools for extracting text and structured data from HWP files.

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Cursor, Write};
use std::path::Path;
use tracing::{debug, error, info};

// ═══════════════════════════════════════════════════════════════════════════
// JSON-RPC 2.0 Types
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcResponse {
    fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MCP Protocol Types
// ═══════════════════════════════════════════════════════════════════════════

const SERVER_NAME: &str = "hwp-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct ServerCapabilities {
    tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
struct ToolsCapability {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[derive(Debug, Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Serialize)]
struct ToolsListResult {
    tools: Vec<Tool>,
}

#[derive(Debug, Serialize)]
struct ToolCallResult {
    content: Vec<ContentItem>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ContentItem {
    #[serde(rename = "text")]
    Text { text: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// MCP Server
// ═══════════════════════════════════════════════════════════════════════════

struct McpServer {
    initialized: bool,
}

impl McpServer {
    fn new() -> Self {
        Self { initialized: false }
    }

    fn handle_request(&mut self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = request.id.clone().unwrap_or(Value::Null);

        // Notification (no id) - no response needed
        if request.id.is_none() {
            match request.method.as_str() {
                "notifications/initialized" => {
                    info!("Client initialized notification received");
                    self.initialized = true;
                }
                _ => {
                    debug!("Unknown notification: {}", request.method);
                }
            }
            return None;
        }

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params),
            "ping" => Ok(json!({})),
            method => Err(anyhow::anyhow!("Method not found: {}", method)),
        };

        Some(match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(e) => JsonRpcResponse::error(id, -32603, e.to_string()),
        })
    }

    fn handle_initialize(&mut self, _params: &Option<Value>) -> Result<Value> {
        info!("Initializing MCP server");

        let result = InitializeResult {
            protocol_version: PROTOCOL_VERSION.to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability {
                    list_changed: false,
                },
            },
            server_info: ServerInfo {
                name: SERVER_NAME.to_string(),
                version: SERVER_VERSION.to_string(),
            },
        };

        Ok(serde_json::to_value(result)?)
    }

    fn handle_tools_list(&self) -> Result<Value> {
        let tools = vec![
            Tool {
                name: "extract_text".to_string(),
                description: "Extract plain text from an HWP document file. Returns the full text content of the document.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute path to the HWP file"
                        },
                        "file_base64": {
                            "type": "string",
                            "description": "Base64-encoded HWP file content (alternative to file_path)"
                        }
                    },
                    "oneOf": [
                        { "required": ["file_path"] },
                        { "required": ["file_base64"] }
                    ]
                }),
            },
            Tool {
                name: "get_structure".to_string(),
                description: "Get structured representation of an HWP document including sections, paragraphs, tables, and metadata. Returns JSON suitable for LLM processing.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute path to the HWP file"
                        },
                        "file_base64": {
                            "type": "string",
                            "description": "Base64-encoded HWP file content (alternative to file_path)"
                        },
                        "include_text": {
                            "type": "boolean",
                            "description": "Include full text in the response (default: true)",
                            "default": true
                        }
                    },
                    "oneOf": [
                        { "required": ["file_path"] },
                        { "required": ["file_base64"] }
                    ]
                }),
            },
            Tool {
                name: "get_info".to_string(),
                description: "Get basic information about an HWP document including version, encryption status, and metadata.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Absolute path to the HWP file"
                        },
                        "file_base64": {
                            "type": "string",
                            "description": "Base64-encoded HWP file content (alternative to file_path)"
                        }
                    },
                    "oneOf": [
                        { "required": ["file_path"] },
                        { "required": ["file_base64"] }
                    ]
                }),
            },
        ];

        Ok(serde_json::to_value(ToolsListResult { tools })?)
    }

    fn handle_tools_call(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing params")?;
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .context("Missing tool name")?;
        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        debug!("Calling tool: {} with args: {:?}", tool_name, arguments);

        let result = match tool_name {
            "extract_text" => self.tool_extract_text(&arguments),
            "get_structure" => self.tool_get_structure(&arguments),
            "get_info" => self.tool_get_info(&arguments),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        };

        match result {
            Ok(text) => Ok(serde_json::to_value(ToolCallResult {
                content: vec![ContentItem::Text { text }],
                is_error: None,
            })?),
            Err(e) => Ok(serde_json::to_value(ToolCallResult {
                content: vec![ContentItem::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            })?),
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Tool Implementations
    // ═══════════════════════════════════════════════════════════════════════

    fn tool_extract_text(&self, args: &Value) -> Result<String> {
        let hwp_data = self.get_hwp_data(args)?;
        let cursor = Cursor::new(hwp_data);

        // HwpTextExtractor::open validates header and fails-fast for encrypted/distribution docs
        let mut extractor =
            hwp_core::HwpTextExtractor::open(cursor).context("Failed to open HWP file")?;

        let text = extractor
            .extract_all_text()
            .context("Failed to extract text")?;

        Ok(text)
    }

    fn tool_get_structure(&self, args: &Value) -> Result<String> {
        let hwp_data = self.get_hwp_data(args)?;
        let include_text = args
            .get("include_text")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // First extract text
        let cursor = Cursor::new(hwp_data.clone());
        let mut extractor =
            hwp_core::HwpTextExtractor::open(cursor).context("Failed to open HWP file")?;
        let text = extractor
            .extract_all_text()
            .context("Failed to extract text")?;

        // Get file info
        let cursor2 = Cursor::new(hwp_data);
        let ole = hwp_core::HwpOleFile::open(cursor2).context("Failed to open HWP file")?;
        let header = ole.header();

        // Build structured response
        // Note: Full document parsing will be added when HwpTextExtractor::extract_document is implemented
        let paragraphs: Vec<Value> = text
            .split('\n')
            .filter(|p| !p.is_empty())
            .map(|p| {
                let text_content = if include_text {
                    p.to_string()
                } else if p.len() > 50 {
                    format!("{}...", &p[..50])
                } else {
                    p.to_string()
                };
                json!({
                    "type": "paragraph",
                    "text": text_content,
                    "paragraph_type": detect_paragraph_type(p),
                })
            })
            .collect();

        let response = json!({
            "metadata": {
                "hwp_version": header.version.to_string(),
                "is_encrypted": header.properties.is_encrypted(),
                "is_distribution": header.properties.is_distribution(),
                "char_count": text.chars().count(),
            },
            "sections": [{
                "index": 0,
                "content": paragraphs,
            }],
        });

        Ok(serde_json::to_string_pretty(&response)?)
    }

    fn tool_get_info(&self, args: &Value) -> Result<String> {
        let hwp_data = self.get_hwp_data(args)?;
        let cursor = Cursor::new(hwp_data);

        let ole = hwp_core::HwpOleFile::open(cursor).context("Failed to open HWP file")?;
        let header = ole.header();

        let info = json!({
            "version": header.version.to_string(),
            "is_encrypted": header.properties.is_encrypted(),
            "is_compressed": header.properties.is_compressed(),
            "is_distribution": header.properties.is_distribution(),
        });

        Ok(serde_json::to_string_pretty(&info)?)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Helper Methods
    // ═══════════════════════════════════════════════════════════════════════

    fn get_hwp_data(&self, args: &Value) -> Result<Vec<u8>> {
        // Try file_path first
        if let Some(path_str) = args.get("file_path").and_then(|v| v.as_str()) {
            let path = Path::new(path_str);
            if !path.exists() {
                anyhow::bail!("File not found: {}", path_str);
            }
            let data = std::fs::read(path)
                .with_context(|| format!("Failed to read file: {}", path_str))?;
            return Ok(data);
        }

        // Try base64
        if let Some(b64) = args.get("file_base64").and_then(|v| v.as_str()) {
            let data = base64::engine::general_purpose::STANDARD
                .decode(b64)
                .context("Invalid base64 encoding")?;
            return Ok(data);
        }

        anyhow::bail!("Either 'file_path' or 'file_base64' must be provided");
    }
}

/// Detect paragraph type from text content
fn detect_paragraph_type(text: &str) -> &'static str {
    let trimmed = text.trim();

    // Numbered list patterns (1. 2. or 1) 2) etc.)
    if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit())
        && (rest.starts_with('.') || rest.starts_with(')'))
    {
        return "numbered_list";
    }

    // Bullet list patterns
    let bullet_chars = ['•', '·', '-', '–', '—', '○', '●', '■', '□', '▪', '▫'];
    if let Some(first_char) = trimmed.chars().next()
        && bullet_chars.contains(&first_char)
    {
        return "bullet_list";
    }

    "body"
}

// ═══════════════════════════════════════════════════════════════════════════
// Main Entry Point
// ═══════════════════════════════════════════════════════════════════════════

fn main() -> Result<()> {
    // Initialize logging to stderr (MCP uses stdout for protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hwp_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    info!("Starting {} v{}", SERVER_NAME, SERVER_VERSION);

    let mut server = McpServer::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to read line: {}", e);
                continue;
            }
        };

        if line.is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                let response = JsonRpcResponse::error(Value::Null, -32700, "Parse error");
                let _ = writeln!(stdout, "{}", serde_json::to_string(&response)?);
                let _ = stdout.flush();
                continue;
            }
        };

        if request.jsonrpc != "2.0" {
            let response = JsonRpcResponse::error(
                request.id.unwrap_or(Value::Null),
                -32600,
                "Invalid JSON-RPC version",
            );
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
            continue;
        }

        if let Some(response) = server.handle_request(request) {
            let response_str = serde_json::to_string(&response)?;
            debug!("Sending: {}", response_str);
            writeln!(stdout, "{}", response_str)?;
            stdout.flush()?;
        }
    }

    info!("Server shutting down");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_initialize_server() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test",
                    "version": "1.0"
                }
            })),
        };

        // Act
        let response = server.handle_request(request);

        // Assert
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.error.is_none());
        assert!(resp.result.is_some());

        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], SERVER_NAME);
    }

    #[test]
    fn test_should_list_tools() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        };

        // Act
        let response = server.handle_request(request);

        // Assert
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.error.is_none());

        let result = resp.result.unwrap();
        let tools = result["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 3);

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(tool_names.contains(&"extract_text"));
        assert!(tool_names.contains(&"get_structure"));
        assert!(tool_names.contains(&"get_info"));
    }

    #[test]
    fn test_should_handle_notification() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None, // Notification - no id
            method: "notifications/initialized".to_string(),
            params: None,
        };

        // Act
        let response = server.handle_request(request);

        // Assert - notifications don't get responses
        assert!(response.is_none());
        assert!(server.initialized);
    }

    #[test]
    fn test_should_return_error_for_unknown_method() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(3)),
            method: "unknown/method".to_string(),
            params: None,
        };

        // Act
        let response = server.handle_request(request);

        // Assert
        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.error.is_some());
        assert!(resp.error.unwrap().message.contains("Method not found"));
    }

    #[test]
    fn test_should_handle_missing_file() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(4)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "extract_text",
                "arguments": {
                    "file_path": "/nonexistent/file.hwp"
                }
            })),
        };

        // Act
        let response = server.handle_request(request);

        // Assert
        assert!(response.is_some());
        let resp = response.unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("File not found")
        );
    }

    #[test]
    fn test_should_require_file_input() {
        // Arrange
        let mut server = McpServer::new();
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(5)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "extract_text",
                "arguments": {}
            })),
        };

        // Act
        let response = server.handle_request(request);

        // Assert
        assert!(response.is_some());
        let resp = response.unwrap();
        let result = resp.result.unwrap();
        assert_eq!(result["isError"], true);
        assert!(
            result["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("file_path")
        );
    }
}
