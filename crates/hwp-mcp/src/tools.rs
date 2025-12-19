//! Tool implementations for HTTP transport.
//!
//! This module provides tool listing and execution for the HTTP server.

use std::io::Cursor;

use anyhow::{Context, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use hwp_core::{
    HwpOleFile, converter::structured::to_semantic_markdown, export::parse_structured_document,
    parser::SectionLimits,
};
use hwp_types::StructuredDocument;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Returns the list of available tools as JSON.
pub fn list_tools() -> Value {
    json!([
        {
            "name": "hwp.inspect",
            "description": "Parse an HWP file and return structured metadata (title, author, section/paragraph/table counts).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "description": "The HWP file (inline base64 or local path)",
                        "oneOf": [
                            {
                                "type": "object",
                                "properties": {
                                    "name": { "type": "string", "description": "Filename" },
                                    "content": { "type": "string", "description": "Base64-encoded file content" }
                                },
                                "required": ["name", "content"]
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "Local file path" }
                                },
                                "required": ["path"]
                            }
                        ]
                    }
                },
                "required": ["file"]
            }
        },
        {
            "name": "hwp.to_markdown",
            "description": "Convert HWP to semantic markdown or plain text.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": {
                        "description": "The HWP file"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["semantic-markdown", "plain"],
                        "default": "semantic-markdown"
                    }
                },
                "required": ["file"]
            }
        },
        {
            "name": "hwp.extract",
            "description": "Extract plain text from HWP (fast path).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": { "description": "The HWP file" }
                },
                "required": ["file"]
            }
        },
        {
            "name": "hwp.to_json",
            "description": "Convert HWP to structured JSON output.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file": { "description": "The HWP file" },
                    "pretty": { "type": "boolean", "default": false }
                },
                "required": ["file"]
            }
        }
    ])
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum ToolFile {
    Inline {
        name: String,
        content: String,
    },
    Path {
        path: String,
        #[serde(default)]
        name: Option<String>,
    },
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
    #[serde(default)]
    pretty: bool,
}

fn default_format() -> String {
    "semantic-markdown".to_string()
}

#[derive(Debug, Serialize)]
struct InspectResult {
    title: String,
    author: String,
    created_at: String,
    is_encrypted: bool,
    is_distributed: bool,
    sections: usize,
    paragraphs: usize,
    tables: usize,
}

fn read_tool_file(file: &ToolFile, max_bytes: usize) -> anyhow::Result<(String, Vec<u8>)> {
    match file {
        ToolFile::Inline { name, content } => {
            let bytes = STANDARD.decode(content).context("Invalid base64 content")?;
            if bytes.len() > max_bytes {
                anyhow::bail!("File exceeds size limit ({} > {})", bytes.len(), max_bytes);
            }
            Ok((name.clone(), bytes))
        }
        ToolFile::Path { path, name } => {
            // Only allow local file access if HWP_ALLOW_LOCAL_FILES=1
            if std::env::var("HWP_ALLOW_LOCAL_FILES").ok().as_deref() != Some("1") {
                anyhow::bail!(
                    "Local file access is disabled. Set HWP_ALLOW_LOCAL_FILES=1 to enable."
                );
            }
            let bytes =
                std::fs::read(path).with_context(|| format!("Failed to read file: {}", path))?;
            if bytes.len() > max_bytes {
                anyhow::bail!("File exceeds size limit ({} > {})", bytes.len(), max_bytes);
            }
            let filename = name.clone().unwrap_or_else(|| {
                std::path::Path::new(path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file.hwp".to_string())
            });
            Ok((filename, bytes))
        }
    }
}

fn parse_hwp(
    bytes: &[u8],
    name: &str,
) -> anyhow::Result<(StructuredDocument, hwp_types::FileHeader)> {
    let cursor = Cursor::new(bytes);
    let ole = HwpOleFile::open(cursor).context("Failed to open OLE container")?;
    let header = ole.header().clone();

    if header.properties.is_encrypted() {
        anyhow::bail!("Encrypted documents are not supported");
    }
    if header.properties.is_distribution() {
        anyhow::bail!("Distribution-only documents are not supported");
    }

    // Re-create cursor for parse_structured_document
    let cursor = Cursor::new(bytes);
    let limits = SectionLimits::default();
    let doc = parse_structured_document(cursor, Some(name.to_string()), limits)
        .with_context(|| format!("Failed to parse document: {}", name))?;

    Ok((doc, header))
}

fn derive_title(name: &str) -> String {
    std::path::Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

fn tool_response(text: String, data: Option<Value>) -> Value {
    let mut content = vec![json!({
        "type": "text",
        "text": text
    })];
    if let Some(d) = data {
        content.push(json!({
            "type": "resource",
            "resource": {
                "uri": "data:application/json",
                "mimeType": "application/json",
                "text": serde_json::to_string(&d).unwrap_or_default()
            }
        }));
    }
    json!({ "content": content })
}

/// Call a tool by name with the given arguments.
pub async fn call_tool(
    name: &str,
    arguments: Value,
    max_file_bytes: usize,
    _max_records: usize,
    _max_sections: usize,
) -> anyhow::Result<Value> {
    match name {
        "hwp.inspect" => {
            let args: InspectArgs = serde_json::from_value(arguments)?;
            let (filename, bytes) = read_tool_file(&args.file, max_file_bytes)?;
            let (doc, header) = parse_hwp(&bytes, &filename)?;

            let result = InspectResult {
                title: doc
                    .metadata
                    .title
                    .clone()
                    .unwrap_or_else(|| derive_title(&filename)),
                author: doc
                    .metadata
                    .author
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                created_at: doc
                    .metadata
                    .created_at
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                is_encrypted: header.properties.is_encrypted(),
                is_distributed: header.properties.is_distribution(),
                sections: doc.sections.len(),
                paragraphs: doc.paragraph_count(),
                tables: doc.table_count(),
            };

            let summary = format!(
                "{}\nsections: {}, paragraphs: {}, tables: {}",
                result.title, result.sections, result.paragraphs, result.tables
            );
            Ok(tool_response(summary, Some(serde_json::to_value(result)?)))
        }
        "hwp.to_markdown" => {
            let args: ConvertArgs = serde_json::from_value(arguments)?;
            let (filename, bytes) = read_tool_file(&args.file, max_file_bytes)?;
            let (doc, _header) = parse_hwp(&bytes, &filename)?;

            let text = match args.format.to_lowercase().as_str() {
                "semantic-markdown" => to_semantic_markdown(&doc),
                "plain" | "plain-text" | "text" => doc.extract_text(),
                other => anyhow::bail!("Unsupported format: {}", other),
            };
            Ok(tool_response(text, None))
        }
        "hwp.extract" => {
            let args: InspectArgs = serde_json::from_value(arguments)?;
            let (filename, bytes) = read_tool_file(&args.file, max_file_bytes)?;
            let (doc, _header) = parse_hwp(&bytes, &filename)?;
            let text = doc.extract_text();
            Ok(tool_response(text, None))
        }
        "hwp.to_json" => {
            let args: JsonArgs = serde_json::from_value(arguments)?;
            let (filename, bytes) = read_tool_file(&args.file, max_file_bytes)?;
            let (doc, _header) = parse_hwp(&bytes, &filename)?;

            let json_str = if args.pretty {
                serde_json::to_string_pretty(&doc)?
            } else {
                serde_json::to_string(&doc)?
            };
            Ok(tool_response(json_str, Some(serde_json::to_value(&doc)?)))
        }
        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}
