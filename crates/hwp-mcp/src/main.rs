use std::{path::Path, sync::Arc};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hwp_core::{
    HwpOleFile,
    converter::structured::to_semantic_markdown,
    export::parse_structured_document,
    parser::{
        DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION, DEFAULT_MAX_RECORDS_PER_SECTION, SectionLimits,
    },
};
use hwp_types::{FileHeader, StructuredDocument};
use mcp_sdk_rs::{
    error::{Error, ErrorCode},
    server::{Server, ServerHandler},
    transport::stdio::StdioTransport,
    types::{
        ClientCapabilities, Implementation, ListToolsResult, MessageContent, ServerCapabilities,
        Tool, ToolAnnotations, ToolResult, ToolSchema,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    sync::mpsc,
};

mod http_server;
mod tools;

const STDIO_BUFFER: usize = 256;

// ─────────────────────────────────────────────────────────────────────────────
// Runtime limits
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeLimits {
    pub max_file_bytes: usize,
    pub max_decompressed_bytes_per_section: usize,
    pub max_records_per_section: usize,
}

tokio::task_local! {
    static RUNTIME_LIMITS: RuntimeLimits;
}

fn env_usize(key: &str) -> Option<usize> {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
}

fn default_limits_from_env() -> RuntimeLimits {
    RuntimeLimits {
        max_file_bytes: env_usize("HWP_MAX_FILE_BYTES").unwrap_or(25 * 1024 * 1024),
        max_decompressed_bytes_per_section: env_usize("HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION")
            .unwrap_or(DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION),
        max_records_per_section: env_usize("HWP_MAX_RECORDS_PER_SECTION")
            .unwrap_or(DEFAULT_MAX_RECORDS_PER_SECTION),
    }
}

pub(crate) fn current_limits() -> RuntimeLimits {
    RUNTIME_LIMITS
        .try_with(|l| *l)
        .unwrap_or_else(|_| default_limits_from_env())
}

pub(crate) async fn scope_limits<F, R>(limits: RuntimeLimits, fut: F) -> R
where
    F: std::future::Future<Output = R>,
{
    RUNTIME_LIMITS.scope(limits, fut).await
}

#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Deserialize)]
struct InspectArgs {
    file: ToolFile,
}

#[derive(Debug, Deserialize)]
struct ConvertArgs {
    file: ToolFile,
    #[serde(default = "default_format")]
    format: String,
}

#[derive(Debug, Deserialize)]
struct JsonArgs {
    file: ToolFile,
    /// Pretty-print JSON (can be large)
    #[serde(default = "default_pretty")]
    pretty: bool,
}

fn default_pretty() -> bool {
    false
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ToolFile {
    /// Inline base64 bytes (default)
    Inline { name: String, content: String },
    /// Local path (only if enabled via env)
    Path {
        path: String,
        #[serde(default)]
        name: Option<String>,
    },
}

#[derive(Debug, Serialize)]
struct InspectHwpResult {
    title: String,
    author: String,
    created_at: String,
    is_encrypted: bool,
    is_distributed: bool,
    sections: usize,
    paragraphs: usize,
    tables: usize,
}

fn current_section_limits() -> SectionLimits {
    let l = current_limits();
    let mut limits = SectionLimits::default();
    limits.max_decompressed_bytes = l.max_decompressed_bytes_per_section;
    limits.max_records = l.max_records_per_section;
    limits
}

fn default_format() -> String {
    "semantic-markdown".to_string()
}

struct HwpServerHandler {
    tools: Vec<Tool>,
}

impl HwpServerHandler {
    fn new() -> Self {
        Self {
            tools: vec![
                inspect_tool(),
                convert_tool(),
                extract_tool(),
                to_json_tool(),
            ],
        }
    }
}

#[async_trait]
impl ServerHandler for HwpServerHandler {
    async fn initialize(
        &self,
        implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, Error> {
        tracing::info!(
            "client_connected" = implementation.name,
            version = implementation.version
        );
        Ok(ServerCapabilities {
            tools: Some(json!({"listChanged": true})),
            ..ServerCapabilities::default()
        })
    }

    async fn shutdown(&self) -> Result<(), Error> {
        tracing::info!("shutdown" = true);
        Ok(())
    }

    async fn handle_method(&self, method: &str, params: Option<Value>) -> Result<Value, Error> {
        match method {
            "tools/list" => {
                let response = ListToolsResult {
                    tools: self.tools.clone(),
                    next_cursor: None,
                };
                Ok(serde_json::to_value(response)?)
            }
            "tools/call" => {
                let params: ToolCallRequest = serde_json::from_value(
                    params.unwrap_or_else(|| json!({"name": "", "arguments": {}})),
                )?;
                self.handle_tool_call(params).await
            }
            _ => Err(Error::protocol(
                ErrorCode::MethodNotFound,
                format!("unsupported method: {method}"),
            )),
        }
    }
}

impl HwpServerHandler {
    async fn handle_tool_call(&self, request: ToolCallRequest) -> Result<Value, Error> {
        match request.name.as_str() {
            "hwp.inspect" => {
                let args: InspectArgs = serde_json::from_value(request.arguments)?;
                let (name, bytes) = read_tool_file_limited(&args.file)?;
                let (doc, header) =
                    parse_hwp(&bytes, &name).map_err(|e| Error::Other(e.to_string()))?;
                let result = InspectHwpResult {
                    title: doc
                        .metadata
                        .title
                        .clone()
                        .unwrap_or_else(|| derive_title(&name)),
                    author: doc
                        .metadata
                        .author
                        .clone()
                        .unwrap_or_else(|| "Unknown Author".to_string()),
                    created_at: doc
                        .metadata
                        .created_at
                        .clone()
                        .unwrap_or_else(default_created_at),
                    is_encrypted: header.properties.is_encrypted(),
                    is_distributed: header.properties.is_distribution(),
                    sections: doc.sections.len(),
                    paragraphs: doc.paragraph_count(),
                    tables: doc.table_count(),
                };
                let summary = format!(
                    "{title}\nsections: {sections}, paragraphs: {paragraphs}, tables: {tables}",
                    title = result.title,
                    sections = result.sections,
                    paragraphs = result.paragraphs,
                    tables = result.tables
                );
                Ok(tool_response(summary, Some(serde_json::to_value(result)?)))
            }
            "hwp.to_markdown" => {
                let args: ConvertArgs = serde_json::from_value(request.arguments)?;
                let (name, bytes) = read_tool_file_limited(&args.file)?;
                let (doc, _header) =
                    parse_hwp(&bytes, &name).map_err(|e| Error::Other(e.to_string()))?;
                let markdown = match args.format.to_lowercase().as_str() {
                    "semantic-markdown" => to_semantic_markdown(&doc),
                    "plain" | "plain-text" | "text" => doc.extract_text(),
                    other => {
                        return Err(Error::protocol(
                            ErrorCode::InvalidParams,
                            format!("unsupported format: {other}"),
                        ));
                    }
                };
                Ok(tool_response(markdown, Some(serde_json::to_value(doc)?)))
            }

            "hwp.extract" => {
                let args: InspectArgs = serde_json::from_value(request.arguments)?;
                let (name, bytes) = read_tool_file_limited(&args.file)?;
                let (doc, _header) =
                    parse_hwp(&bytes, &name).map_err(|e| Error::Other(e.to_string()))?;
                Ok(tool_response(
                    doc.extract_text(),
                    Some(serde_json::to_value(doc)?),
                ))
            }
            "hwp.to_json" => {
                let args: JsonArgs = serde_json::from_value(request.arguments)?;
                let (name, bytes) = read_tool_file_limited(&args.file)?;
                let (doc, _header) =
                    parse_hwp(&bytes, &name).map_err(|e| Error::Other(e.to_string()))?;
                let json_text = if args.pretty {
                    serde_json::to_string_pretty(&doc).unwrap_or_else(|_| "{}".to_string())
                } else {
                    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".to_string())
                };
                Ok(tool_response(json_text, Some(serde_json::to_value(doc)?)))
            }

            other => Err(Error::protocol(
                ErrorCode::MethodNotFound,
                format!("unknown tool: {other}"),
            )),
        }
    }
}

fn inspect_tool() -> Tool {
    Tool {
        name: "hwp.inspect".to_string(),
        description: "Inspect a HWP file and return document metadata and stats.".to_string(),
        input_schema: Some(ToolSchema {
            properties: Some(json!({
                "file": {
                    "type": "object",
                    "description": "HWP payload encoded as base64 (content) and logical name.",
                    "properties": {
                        "name": {"type": "string", "description": "Logical filename"},
                        "content": {"type": "string", "description": "base64 encoded bytes", "contentEncoding": "base64"},
                        "path": {"type": "string", "description": "Local file path (only if HWP_ALLOW_PATH_INPUT=1)"}
                    },
                    "oneOf": [
                      {"required": ["name", "content"]},
                      {"required": ["path"]}
                    ]
                }
            })),
            required: Some(vec!["file".to_string()]),
        }),
        annotations: Some(ToolAnnotations {
            title: Some("Inspect HWP".to_string()),
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
    }
}

fn convert_tool() -> Tool {
    Tool {
        name: "hwp.to_markdown".to_string(),
        description: "Convert HWP content to semantic markdown preserving nested tables."
            .to_string(),
        input_schema: Some(ToolSchema {
            properties: Some(json!({
                "file": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "content": {"type": "string", "contentEncoding": "base64"},
                        "path": {"type": "string"}
                    },
                    "oneOf": [
                      {"required": ["name", "content"]},
                      {"required": ["path"]}
                    ]
                },
                "format": {
                    "type": "string",
                    "enum": ["semantic-markdown", "plain"],
                    "default": "semantic-markdown"
                }
            })),
            required: Some(vec!["file".to_string()]),
        }),
        annotations: Some(ToolAnnotations {
            title: Some("Convert HWP".to_string()),
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
    }
}

fn tool_response(summary: String, structured: Option<Value>) -> Value {
    let result = ToolResult {
        content: vec![MessageContent::Text { text: summary }],
        structured_content: structured,
    };
    serde_json::to_value(result).expect("tool result serializable")
}

fn extract_tool() -> Tool {
    Tool {
        name: "hwp.extract".to_string(),
        description: "Extract plain text from a HWP file (fast path).".to_string(),
        input_schema: Some(ToolSchema {
            properties: Some(json!({
                "file": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "content": {"type": "string", "contentEncoding": "base64"},
                        "path": {"type": "string"}
                    },
                    "oneOf": [
                      {"required": ["name", "content"]},
                      {"required": ["path"]}
                    ]
                }
            })),
            required: Some(vec!["file".to_string()]),
        }),
        annotations: Some(ToolAnnotations {
            title: Some("Extract HWP Text".to_string()),
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
    }
}

fn to_json_tool() -> Tool {
    Tool {
        name: "hwp.to_json".to_string(),
        description: "Convert a HWP file into structured JSON (text output + structured payload)."
            .to_string(),
        input_schema: Some(ToolSchema {
            properties: Some(json!({
                "file": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "content": {"type": "string", "contentEncoding": "base64"},
                        "path": {"type": "string"}
                    },
                    "oneOf": [
                      {"required": ["name", "content"]},
                      {"required": ["path"]}
                    ]
                },
                "pretty": {
                    "type": "boolean",
                    "default": false
                }
            })),
            required: Some(vec!["file".to_string()]),
        }),
        annotations: Some(ToolAnnotations {
            title: Some("Convert HWP to JSON".to_string()),
            read_only_hint: Some(false),
            destructive_hint: Some(false),
            idempotent_hint: Some(true),
            open_world_hint: None,
        }),
    }
}

fn allow_path_input() -> bool {
    std::env::var("HWP_ALLOW_PATH_INPUT")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn path_basedir() -> Option<std::path::PathBuf> {
    std::env::var("HWP_PATH_BASEDIR")
        .ok()
        .map(std::path::PathBuf::from)
}

fn read_tool_file(file: &ToolFile) -> Result<(String, Vec<u8>), Error> {
    match file {
        ToolFile::Inline { name, content } => {
            let bytes = STANDARD
                .decode(content)
                .map_err(|e| Error::protocol(ErrorCode::InvalidParams, e.to_string()))?;
            Ok((name.clone(), bytes))
        }
        ToolFile::Path { path, name } => {
            if !allow_path_input() {
                return Err(Error::protocol(
                    ErrorCode::InvalidParams,
                    "path input is disabled (set HWP_ALLOW_PATH_INPUT=1)".to_string(),
                ));
            }
            let p = std::path::PathBuf::from(path);
            let canon = std::fs::canonicalize(&p).map_err(|e| {
                Error::protocol(ErrorCode::InvalidParams, format!("invalid path: {e}"))
            })?;

            if let Some(base) = path_basedir() {
                let base = std::fs::canonicalize(base).map_err(|e| {
                    Error::protocol(ErrorCode::InvalidParams, format!("invalid basedir: {e}"))
                })?;
                if !canon.starts_with(&base) {
                    return Err(Error::protocol(
                        ErrorCode::InvalidParams,
                        "path is outside of HWP_PATH_BASEDIR".to_string(),
                    ));
                }
            }

            let bytes = std::fs::read(&canon).map_err(|e| {
                Error::protocol(ErrorCode::InvalidParams, format!("read failed: {e}"))
            })?;
            let logical_name = name
                .clone()
                .or_else(|| {
                    canon
                        .file_name()
                        .and_then(|v| v.to_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "input.hwp".to_string());
            Ok((logical_name, bytes))
        }
    }
}

fn unsupported_hwpx_error() -> String {
    "UNSUPPORTED_FORMAT: HWPX (.hwpx) is not supported yet. Supported: HWP v5 (.hwp, OLE/CFB)."
        .to_string()
}

fn is_hwpx_name(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".hwpx")
}

/// HWPX files are ZIP packages; they typically start with PK\x03\x04.
fn is_hwpx_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x50 && bytes[1] == 0x4B && bytes[2] == 0x03 && bytes[3] == 0x04
}

fn read_tool_file_limited(file: &ToolFile) -> Result<(String, Vec<u8>), Error> {
    let (name, bytes) = read_tool_file(file)?;
    // Explicitly block HWPX early with a consistent error.
    if is_hwpx_name(&name) || is_hwpx_magic(&bytes) {
        return Err(Error::protocol(
            ErrorCode::InvalidParams,
            unsupported_hwpx_error(),
        ));
    }
    let max = current_limits().max_file_bytes;
    if bytes.len() > max {
        return Err(Error::protocol(
            ErrorCode::InvalidParams,
            format!("file too large: {} bytes (max: {} bytes)", bytes.len(), max),
        ));
    }
    Ok((name, bytes))
}

fn derive_title(name: &str) -> String {
    Path::new(name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name)
        .to_string()
}

fn default_created_at() -> String {
    "unknown".to_string()
}

fn parse_hwp(data: &[u8], file_name: &str) -> Result<(StructuredDocument, FileHeader)> {
    let header = {
        let cursor = std::io::Cursor::new(data);
        let mut ole = HwpOleFile::open(cursor).context("failed to open OLE container")?;
        ole.header().clone()
    };

    let limits = current_section_limits();
    let title = Some(derive_title(file_name));
    let doc = parse_structured_document(std::io::Cursor::new(data), title, limits)
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok((doc, header))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::try_init().ok();

    let transport = std::env::var("HWP_MCP_TRANSPORT").unwrap_or_else(|_| "stdio".to_string());
    let handler = Arc::new(HwpServerHandler::new());

    match transport.as_str() {
        "http" => http_server::serve()
            .await
            .map_err(|e| Error::protocol(ErrorCode::InternalError, e.to_string())),
        _ => run_stdio(handler).await,
    }
}

async fn run_stdio(handler: Arc<HwpServerHandler>) -> Result<(), Error> {
    let (incoming_tx, incoming_rx) = mpsc::channel::<String>(STDIO_BUFFER);
    let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<String>(STDIO_BUFFER);

    // Reader task (stdin -> transport)
    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if incoming_tx.send(line.trim().to_string()).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    tracing::error!("stdin_read_error" = %err);
                    break;
                }
            }
        }
    });

    // Writer task (transport -> stdout)
    tokio::spawn(async move {
        let stdout = tokio::io::stdout();
        let mut writer = BufWriter::new(stdout);
        while let Some(message) = outgoing_rx.recv().await {
            if writer.write_all(message.as_bytes()).await.is_err() {
                break;
            }
            if writer.write_all(b"\n").await.is_err() {
                break;
            }
            if writer.flush().await.is_err() {
                break;
            }
        }
    });

    let transport = Arc::new(StdioTransport::new(incoming_rx, outgoing_tx));
    let server = Server::new(transport, handler);
    server.start().await
}
