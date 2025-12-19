use std::{path::Path, sync::Arc};

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::Utc;
use hwp_core::{
    HwpOleFile,
    converter::structured::to_semantic_markdown,
    parser::{decompress_section, record_nom::RecordIteratorNom},
};
use hwp_types::{
    CellBlock, FileHeader, HwpError, ParagraphType, RecordTag, SemanticParagraph, SemanticTable,
    StructuredDocument, StructuredParagraph, StructuredSection, StructuredTable,
    StructuredTableCell,
};
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

const STDIO_BUFFER: usize = 256;

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
struct ToolFile {
    name: String,
    /// Base64 encoded bytes
    content: String,
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

fn default_format() -> String {
    "semantic-markdown".to_string()
}

struct HwpServerHandler {
    tools: Vec<Tool>,
}

impl HwpServerHandler {
    fn new() -> Self {
        Self {
            tools: vec![inspect_tool(), convert_tool()],
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
                let bytes = decode_file(&args.file)?;
                let (doc, header) = build_structured_document(&bytes, &args.file.name)
                    .map_err(|e| Error::Other(e.to_string()))?;
                let result = InspectHwpResult {
                    title: doc
                        .metadata
                        .title
                        .clone()
                        .unwrap_or_else(|| derive_title(&args.file.name)),
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
                let bytes = decode_file(&args.file)?;
                let (doc, _header) = build_structured_document(&bytes, &args.file.name)
                    .map_err(|e| Error::Other(e.to_string()))?;
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
                        "name": {"type": "string"},
                        "content": {
                            "type": "string",
                            "description": "base64 encoded bytes",
                            "contentEncoding": "base64"
                        }
                    },
                    "required": ["name", "content"]
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
                        "content": {
                            "type": "string",
                            "contentEncoding": "base64"
                        }
                    },
                    "required": ["name", "content"]
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

fn decode_file(file: &ToolFile) -> Result<Vec<u8>, Error> {
    STANDARD
        .decode(&file.content)
        .map_err(|e| Error::protocol(ErrorCode::InvalidParams, e.to_string()))
}

fn derive_title(name: &str) -> String {
    Path::new(name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(name)
        .to_string()
}

fn default_created_at() -> String {
    Utc::now().to_rfc3339()
}

fn build_structured_document(
    data: &[u8],
    file_name: &str,
) -> Result<(StructuredDocument, FileHeader)> {
    let cursor = std::io::Cursor::new(data);
    let mut ole = HwpOleFile::open(cursor).context("failed to open OLE container")?;
    let header = ole.header().clone();

    let mut doc = StructuredDocument::new();
    doc.metadata.title = Some(derive_title(file_name));
    doc.metadata.author = Some("Unknown Author".into());
    doc.metadata.created_at = Some(default_created_at());
    doc.metadata.is_encrypted = header.properties.is_encrypted();
    doc.metadata.is_distribution = header.properties.is_distribution();

    let mut section_idx = 0;
    loop {
        match ole.read_section(section_idx) {
            Ok(compressed) => {
                let decompressed =
                    decompress_section(&compressed).context("failed to decompress section")?;
                let mut structured_section = StructuredSection::new(section_idx);

                for record in RecordIteratorNom::new(&decompressed) {
                    let record = record.map_err(|e| anyhow!(e.to_string()))?;
                    match record.tag() {
                        RecordTag::ParaText => {
                            let semantic = SemanticParagraph::try_from(&record)
                                .map_err(|e| anyhow!(e.to_string()))?;
                            if let Some(paragraph) = semantic_paragraph_to_structured(semantic) {
                                structured_section.add_paragraph(paragraph);
                            }
                        }
                        RecordTag::Table => {
                            let semantic = SemanticTable::try_from(&record)
                                .map_err(|e| anyhow!(e.to_string()))?;
                            let table = semantic_table_to_structured(&semantic);
                            structured_section.add_table(table);
                        }
                        _ => {}
                    }
                }

                if !structured_section.content.is_empty() {
                    doc.add_section(structured_section);
                }
                section_idx += 1;
            }
            Err(HwpError::NotFound(_)) => break,
            Err(err) => return Err(err.into()),
        }
    }

    if doc.sections.is_empty() {
        let mut fallback = StructuredSection::new(0);
        fallback.add_paragraph(StructuredParagraph::from_text(
            "본문을 추출하지 못했습니다.",
        ));
        doc.add_section(fallback);
    }

    Ok((doc, header))
}

fn semantic_paragraph_to_structured(
    semantic: SemanticParagraph<'_>,
) -> Option<StructuredParagraph> {
    let text = semantic.text.to_string();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut paragraph = StructuredParagraph::from_text(trimmed.to_string());
    if let Some(header) = semantic.header {
        if (1..=6).contains(&header.style_id) {
            paragraph.paragraph_type = ParagraphType::Heading {
                level: header.style_id.min(6),
            };
        }
    }

    Some(paragraph)
}

fn semantic_table_to_structured(table: &SemanticTable<'_>) -> StructuredTable {
    let mut structured = StructuredTable::new(table.rows as usize, table.cols as usize);
    if table.rows > 0 {
        structured.header_rows = 1;
    }

    let mut rows: Vec<Vec<StructuredTableCell>> = vec![Vec::new(); table.rows as usize];
    for cell in &table.cells {
        if cell.address.row >= rows.len() {
            continue;
        }

        let mut structured_cell = StructuredTableCell::from_text("");
        structured_cell.blocks.clear();
        structured_cell = structured_cell.with_position(cell.address.row, cell.address.col);
        structured_cell.col_span = cell.col_span as usize;
        structured_cell.row_span = cell.row_span as usize;
        structured_cell.is_header = cell.address.row == 0;

        for paragraph in &cell.paragraphs {
            let text = paragraph.text.to_string();
            if text.trim().is_empty() {
                continue;
            }
            structured_cell.push_block(CellBlock::Paragraph(StructuredParagraph::from_text(text)));
        }

        if structured_cell.blocks.is_empty() {
            structured_cell.push_block(CellBlock::RawText {
                text: format!("cell({},{})", cell.address.row + 1, cell.address.col + 1),
            });
        }

        rows[cell.address.row].push(structured_cell);
    }

    for row in rows {
        if row.is_empty() {
            continue;
        }
        structured.add_row(row);
    }

    structured
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::try_init().ok();

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
    let handler = Arc::new(HwpServerHandler::new());
    let server = Server::new(transport, handler);
    server.start().await
}
