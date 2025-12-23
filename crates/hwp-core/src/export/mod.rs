// crates/hwp-core/src/export/mod.rs
//!
//! High-level export helpers:
//! - HWP (OLE) -> StructuredDocument (JSON-friendly) and semantic markdown
//!
//! This is intentionally conservative: it focuses on deterministic output,
//! fail-fast on encrypted/distribution documents (handled by FileHeader::validate),
//! and resource limits (decompression/record count).

use std::io::{Cursor, Read, Seek};

use crate::parser::{
    HwpOleFile, SectionLimits, chart::parse_chart_contents, decompress_section_with_limits,
    parse_docinfo, record_nom::RecordIteratorNom,
};
use hwp_types::{
    BinData, CellBlock, ContentBlock, HwpError, ParagraphType, RecordTag, SemanticParagraph,
    SemanticTable, StructuredChart, StructuredDocument, StructuredParagraph, StructuredSection,
    StructuredTable, StructuredTableCell,
};
use cfb::CompoundFile;

const MAX_HEADING_LEVEL: u8 = 6;
const OLE_CTRL_ID: u32 = u32::from_le_bytes(*b"$ole");

const HWP_ARTIFACT_TOKENS: [&str; 11] = [
    "氠瑢",
    "湯慴",
    "浵╦",
    "浵ࡦ",
    "†普",
    "捤獥汤捯湰灧",
    "ᑋĀ",
    "฿Ā",
    "⸓Ā",
    "ዯĀ",
    "ቼĀ",
];

#[derive(Clone, Copy)]
enum TableParseMode {
    Strict,
    Placeholder,
}

/// Parse an HWP file into a StructuredDocument.
///
/// - Extracts metadata from HwpSummaryInformation stream if available.
/// - Falls back to caller-provided title if not found in document.
pub fn parse_structured_document<F: Read + Seek>(
    reader: F,
    title: Option<String>,
    limits: SectionLimits,
) -> Result<StructuredDocument, HwpError> {
    parse_structured_document_with_mode(reader, title, limits, TableParseMode::Strict)
}

/// Parse an HWP file into a StructuredDocument, tolerating table parse failures.
///
/// - Table parse errors are replaced with placeholder paragraphs.
pub fn parse_structured_document_lenient<F: Read + Seek>(
    reader: F,
    title: Option<String>,
    limits: SectionLimits,
) -> Result<StructuredDocument, HwpError> {
    parse_structured_document_with_mode(reader, title, limits, TableParseMode::Placeholder)
}

fn parse_structured_document_with_mode<F: Read + Seek>(
    reader: F,
    title: Option<String>,
    limits: SectionLimits,
    table_parse_mode: TableParseMode,
) -> Result<StructuredDocument, HwpError> {
    let mut ole = HwpOleFile::open(reader)?;
    let header = ole.header().clone();

    // Read document summary information (title, author, etc.)
    let summary = ole.read_summary_info();
    let bin_data = match ole.read_doc_info() {
        Ok(data) => parse_docinfo(&data)
            .map(|docinfo| docinfo.bin_data)
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let mut doc = StructuredDocument::new();
    // Prefer title from document metadata, fall back to caller-provided title
    doc.metadata.title = summary.title.or(title);
    doc.metadata.author = summary.author;
    doc.metadata.created_at = summary.created_at;
    doc.metadata.is_encrypted = header.properties.is_encrypted();
    doc.metadata.is_distribution = header.properties.is_distribution();

    let is_compressed = header.properties.is_compressed();

    let mut section_idx = 0usize;
    loop {
        if section_idx >= limits.max_sections {
            return Err(HwpError::SizeLimitExceeded(format!(
                "Section count exceeds limit: {} > {}",
                section_idx, limits.max_sections
            )));
        }
        match ole.read_section(section_idx) {
            Ok(section_bytes) => {
                let decompressed = if is_compressed {
                    decompress_section_with_limits(&section_bytes, limits.max_decompressed_bytes)?
                } else if section_bytes.len() > limits.max_decompressed_bytes {
                    return Err(HwpError::SizeLimitExceeded(format!(
                        "Uncompressed section exceeds limit: {} > {} bytes",
                        section_bytes.len(),
                        limits.max_decompressed_bytes
                    )));
                } else {
                    section_bytes
                };

                let mut structured_section = StructuredSection::new(section_idx);
                let mut record_count: usize = 0;

                let mut pending_ole_chart = false;

                for record in RecordIteratorNom::new(&decompressed) {
                    let record = record.map_err(|e| HwpError::ParseError(e.to_string()))?;
                    record_count += 1;
                    if record_count > limits.max_records {
                        return Err(HwpError::SizeLimitExceeded(format!(
                            "Section record count exceeds limit: {} > {}",
                            record_count, limits.max_records
                        )));
                    }

                    match record.tag() {
                        RecordTag::CtrlHeader => {
                            pending_ole_chart = is_ole_ctrl_header(record.data);
                        }
                        RecordTag::ShapeComponentOle => {
                            if pending_ole_chart {
                                let chart = build_chart_placeholder(
                                    record.data,
                                    &bin_data,
                                    &mut ole,
                                );
                                structured_section.add_content(ContentBlock::Chart(chart));
                                pending_ole_chart = false;
                            }
                        }
                        RecordTag::ParaText => {
                            let semantic = SemanticParagraph::try_from(&record)
                                .map_err(|e| HwpError::ParseError(e.to_string()))?;
                            if let Some(paragraph) = semantic_paragraph_to_structured(semantic) {
                                structured_section.add_paragraph(paragraph);
                            }
                        }
                        RecordTag::Table => match SemanticTable::try_from(&record) {
                            Ok(semantic) => {
                                let table = semantic_table_to_structured(&semantic);
                                structured_section.add_table(table);
                            }
                            Err(e) => match table_parse_mode {
                                TableParseMode::Strict => {
                                    return Err(HwpError::ParseError(e.to_string()))
                                }
                                TableParseMode::Placeholder => {
                                    if let Some((rows, cols)) = parse_table_dimensions(record.data)
                                    {
                                        let semantic = SemanticTable::new(rows, cols);
                                        let table = semantic_table_to_structured(&semantic);
                                        structured_section.add_table(table);
                                    } else {
                                        structured_section.add_paragraph(
                                            StructuredParagraph::from_text(format!(
                                                "[표 파싱 실패: {}]",
                                                e
                                            )),
                                        );
                                    }
                                }
                            },
                        },
                        _ => {}
                    }
                }

                // Skip empty sections to keep output stable and compact
                if !structured_section.content.is_empty() {
                    doc.add_section(structured_section);
                }

                section_idx += 1;
            }
            Err(HwpError::NotFound(_)) => break,
            Err(err) => return Err(err),
        }
    }

    if doc.sections.is_empty() {
        let mut fallback = StructuredSection::new(0);
        fallback.add_paragraph(StructuredParagraph::from_text(
            "본문을 추출하지 못했습니다.",
        ));
        doc.add_section(fallback);
    }

    Ok(doc)
}

fn semantic_paragraph_to_structured(
    semantic: SemanticParagraph<'_>,
) -> Option<StructuredParagraph> {
    let text = sanitize_text(semantic.text.as_ref());
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut paragraph = StructuredParagraph::from_text(trimmed.to_string());
    if let Some(header) = semantic.header && header.style_id >= 1 {
        paragraph.paragraph_type = ParagraphType::Heading {
            level: header.style_id.min(MAX_HEADING_LEVEL),
        };
    }

    Some(paragraph)
}

fn semantic_table_to_structured(semantic: &SemanticTable<'_>) -> StructuredTable {
    let row_count = semantic.rows as usize;
    let col_count = semantic.cols as usize;

    let mut table = StructuredTable::new(row_count.max(1), col_count.max(1));
    table.header_rows = 0;

    // Initialize grid with default cells (position will be assigned by add_row)
    let mut grid: Vec<Vec<StructuredTableCell>> =
        vec![vec![StructuredTableCell::default(); col_count.max(1)]; row_count.max(1)];

    for cell in &semantic.cells {
        let r = cell.address.row.min(row_count.saturating_sub(1));
        let c = cell.address.col.min(col_count.saturating_sub(1));

        let mut structured_cell = StructuredTableCell::default()
            .with_position(r, c)
            .with_span(cell.col_span as usize, cell.row_span as usize);

        structured_cell.is_header = r == 0;
        if structured_cell.is_header {
            table.header_rows = table.header_rows.max(1);
        }

        // Replace default blocks with extracted content
        structured_cell.blocks.clear();

        for paragraph in &cell.paragraphs {
            let text = sanitize_text(paragraph.text.as_ref());
            if text.trim().is_empty() {
                continue;
            }
            structured_cell.push_block(CellBlock::Paragraph(StructuredParagraph::from_text(text)));
        }

        grid[r][c] = structured_cell;
    }

    for row in grid {
        table.add_row(row);
    }

    table
}

fn sanitize_text(input: &str) -> String {
    let mut cleaned: String = input.chars().filter(|c| !c.is_control()).collect();
    for token in HWP_ARTIFACT_TOKENS {
        cleaned = cleaned.replace(token, "");
    }
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_ole_ctrl_header(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }
    let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    ctrl_id == OLE_CTRL_ID
}

fn build_chart_placeholder<F: Read + Seek>(
    data: &[u8],
    bin_data: &[BinData],
    ole: &mut HwpOleFile<F>,
) -> StructuredChart {
    let mut chart = StructuredChart {
        bin_data_id: find_bin_data_id(data, bin_data),
        ..StructuredChart::default()
    };

    if let Some(bin_id) = chart.bin_data_id {
        if let Some(bin) = bin_data.get(bin_id as usize) {
            if let Some(stream) = read_bin_data_stream(ole, bin) {
                if let Some((stream_type, contents)) = extract_chart_stream(&stream) {
                    chart.stream_type = Some(stream_type.clone());
                    if stream_type == "contents" {
                        if let Some(parsed) = parse_chart_contents(&contents) {
                            chart.title = parsed.title;
                            chart.chart_type = parsed.chart_type;
                            chart.data_grid = parsed.data_grid;
                        } else {
                            chart.note = Some("chart contents parse failed".to_string());
                        }
                    } else {
                        chart.note = Some("OOXML chart parsing not implemented".to_string());
                    }
                } else {
                    chart.note = Some("chart stream not found".to_string());
                }
            } else {
                chart.note = Some("bin data stream not found".to_string());
            }
        }
    } else {
        chart.note = Some("bin data id not found".to_string());
    }

    chart
}

fn find_bin_data_id(data: &[u8], bin_data: &[BinData]) -> Option<u16> {
    let max_id = u16::try_from(bin_data.len()).ok()?;
    let mut fallback = None;

    let mut idx = 0usize;
    while idx + 1 < data.len() {
        let candidate = u16::from_le_bytes([data[idx], data[idx + 1]]);
        if candidate < max_id
            && let Some(bin) = bin_data.get(candidate as usize)
            && (bin.is_ole() || bin.storage_type.is_storage())
        {
            return Some(candidate);
        }
        if candidate < max_id && fallback.is_none() {
            fallback = Some(candidate);
        }
        idx += 2;
    }

    fallback
}

fn read_bin_data_stream<F: Read + Seek>(
    ole: &mut HwpOleFile<F>,
    bin_data: &BinData,
) -> Option<Vec<u8>> {
    let ext = bin_data.extension.trim();
    if ext.is_empty() {
        return None;
    }

    let stream_name = format!("/BinData/BIN{:04X}.{}", bin_data.id + 1, ext);
    if !ole.has_stream(&stream_name) {
        return None;
    }
    ole.read(&stream_name).ok()
}

fn extract_chart_stream(data: &[u8]) -> Option<(String, Vec<u8>)> {
    const OLE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    if data.len() < OLE_MAGIC.len() || data[..OLE_MAGIC.len()] != OLE_MAGIC {
        return None;
    }

    let cursor = Cursor::new(data);
    let mut cfb = CompoundFile::open(cursor).ok()?;

    if let Some(contents) = read_cfb_stream(&mut cfb, "Contents") {
        return Some(("contents".to_string(), contents));
    }
    if let Some(contents) = read_cfb_stream(&mut cfb, "OOXML.ChartContents")
        .or_else(|| read_cfb_stream(&mut cfb, "OOXML/ChartContents"))
    {
        return Some(("ooxml".to_string(), contents));
    }

    None
}

fn read_cfb_stream<F: Read + Seek>(
    cfb: &mut CompoundFile<F>,
    name: &str,
) -> Option<Vec<u8>> {
    let stream_name = if name.starts_with('/') {
        name.to_string()
    } else {
        format!("/{}", name)
    };
    if !cfb.is_stream(&stream_name) {
        return None;
    }
    let mut stream = cfb.open_stream(&stream_name).ok()?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn parse_table_dimensions(data: &[u8]) -> Option<(u16, u16)> {
    if data.len() < 8 {
        return None;
    }
    let rows = u16::from_le_bytes([data[4], data[5]]);
    let cols = u16::from_le_bytes([data[6], data[7]]);
    if rows == 0 || cols == 0 {
        return None;
    }
    Some((rows, cols))
}

#[cfg(test)]
mod tests {
    use hwp_types::{ParagraphHeader, ParagraphType, SemanticParagraph};
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_should_convert_plain_text_paragraph() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader::default()),
            text: Cow::Borrowed("이것은 일반 텍스트입니다."),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_some());
        let structured = result.unwrap();
        assert_eq!(structured.plain_text(), "이것은 일반 텍스트입니다.");
        assert_eq!(structured.paragraph_type, ParagraphType::Body);
    }

    #[test]
    fn test_should_return_none_for_whitespace_paragraph() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader::default()),
            text: Cow::Borrowed("   \t\n "),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn test_should_strip_hwp_artifact_tokens() {
        let input = "氠瑢 테스트 ᑋĀ 데이터   \t  셋 †普 157,455浵ࡦ건";
        let result = sanitize_text(input);
        assert_eq!(result, "테스트 데이터 셋 157,455건");
    }

    #[test]
    fn test_should_parse_table_dimensions_from_record_data() {
        let mut data = Vec::new();
        data.extend_from_slice(&0x0001u32.to_le_bytes());
        data.extend_from_slice(&3u16.to_le_bytes());
        data.extend_from_slice(&4u16.to_le_bytes());

        let dims = parse_table_dimensions(&data);
        assert_eq!(dims, Some((3, 4)));
    }

    #[test]
    fn test_should_return_none_for_empty_paragraph() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader::default()),
            text: Cow::Borrowed(""),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn test_should_convert_heading_paragraph() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader {
                style_id: 1,
                ..Default::default()
            }),
            text: Cow::Borrowed("제목 1"),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_some());
        let structured = result.unwrap();
        assert_eq!(structured.plain_text(), "제목 1");
        assert_eq!(
            structured.paragraph_type,
            ParagraphType::Heading { level: 1 }
        );
    }

    #[test]
    fn test_should_cap_heading_level_at_6() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader {
                style_id: 7, // Level > 6
                ..Default::default()
            }),
            text: Cow::Borrowed("너무 깊은 제목"),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_some());
        let structured = result.unwrap();
        assert_eq!(
            structured.paragraph_type,
            ParagraphType::Heading { level: 6 }
        );
    }

    #[test]
    fn test_should_treat_style_id_0_as_body() {
        // Arrange
        let semantic = SemanticParagraph {
            header: Some(ParagraphHeader {
                style_id: 0,
                ..Default::default()
            }),
            text: Cow::Borrowed("본문 스타일"),
            spans: vec![],
        };

        // Act
        let result = semantic_paragraph_to_structured(semantic);

        // Assert
        assert!(result.is_some());
        let structured = result.unwrap();
        assert_eq!(structured.paragraph_type, ParagraphType::Body);
    }

    #[test]
    fn test_should_convert_simple_table() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 1,
            cols: 1,
            cells: vec![hwp_types::SemanticTableCell {
                address: hwp_types::CellCoordinate { row: 0, col: 0 },
                col_span: 1,
                row_span: 1,
                size: (0, 0),
                field_name: Cow::Borrowed(""),
                paragraphs: vec![SemanticParagraph {
                    header: None,
                    text: Cow::Borrowed("Cell 1"),
                    spans: vec![],
                }],
                nested_tables: vec![],
            }],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.row_count, 1);
        assert_eq!(result.col_count, 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].len(), 1);
        assert_eq!(result.rows[0][0].plain_text(), "Cell 1");
    }

    #[test]
    fn test_should_handle_empty_table() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 0,
            cols: 0,
            cells: vec![],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.row_count, 1);
        assert_eq!(result.col_count, 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].len(), 1);
        assert!(result.rows[0][0].plain_text().is_empty());
    }

    #[test]
    fn test_should_handle_merged_cells() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 2,
            cols: 2,
            cells: vec![
                // Row 0, Col 0, spans 2 columns
                hwp_types::SemanticTableCell {
                    col_span: 2,
                    paragraphs: vec![SemanticParagraph::new_text("Merged Cell")],
                    ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 0, col: 0 })
                },
                // Row 1, Col 0
                hwp_types::SemanticTableCell {
                    paragraphs: vec![SemanticParagraph::new_text("Cell A")],
                    ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 1, col: 0 })
                },
                // Row 1, Col 1
                hwp_types::SemanticTableCell {
                    paragraphs: vec![SemanticParagraph::new_text("Cell B")],
                    ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 1, col: 1 })
                },
            ],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.row_count, 2);
        assert_eq!(result.col_count, 2);
        assert_eq!(result.rows.len(), 2);
        // The first row has 2 cells in the grid, but the first one has a colspan of 2
        assert_eq!(result.rows[0][0].col_span, 2);
        assert_eq!(result.rows[0][0].plain_text(), "Merged Cell");
        // The second cell in the first row should still exist but might be marked differently
        // Depending on the implementation, it might be an empty cell or not present in the final output row.
        // The current implementation fills the grid, let's check the grid.
        assert_eq!(result.rows[0].len(), 2);
        assert_eq!(result.rows[1].len(), 2);
        assert_eq!(result.rows[1][0].plain_text(), "Cell A");
        assert_eq!(result.rows[1][1].plain_text(), "Cell B");
    }

    #[test]
    fn test_should_mark_header_row() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 2,
            cols: 1,
            cells: vec![
                hwp_types::SemanticTableCell {
                    paragraphs: vec![SemanticParagraph::new_text("Header")],
                    ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 0, col: 0 })
                },
                hwp_types::SemanticTableCell {
                    paragraphs: vec![SemanticParagraph::new_text("Data")],
                    ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 1, col: 0 })
                },
            ],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.header_rows, 1);
        assert!(result.rows[0][0].is_header);
        assert!(!result.rows[1][0].is_header);
    }

    #[test]
    fn test_should_handle_multiple_paragraphs_in_cell() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 1,
            cols: 1,
            cells: vec![hwp_types::SemanticTableCell {
                paragraphs: vec![
                    SemanticParagraph::new_text("Line 1"),
                    SemanticParagraph::new_text("Line 2"),
                ],
                ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 0, col: 0 })
            }],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.rows[0][0].blocks.len(), 2);
        match &result.rows[0][0].blocks[0] {
            CellBlock::Paragraph(p) => assert_eq!(p.plain_text(), "Line 1"),
            _ => panic!("Expected a paragraph block"),
        }
        match &result.rows[0][0].blocks[1] {
            CellBlock::Paragraph(p) => assert_eq!(p.plain_text(), "Line 2"),
            _ => panic!("Expected a paragraph block"),
        }
    }

    #[test]
    fn test_should_skip_empty_paragraph_in_cell() {
        // Arrange
        let semantic_table = SemanticTable {
            properties: 0,
            rows: 1,
            cols: 1,
            cells: vec![hwp_types::SemanticTableCell {
                paragraphs: vec![
                    SemanticParagraph::new_text(""),
                    SemanticParagraph::new_text("Content"),
                    SemanticParagraph::new_text("   "),
                ],
                ..hwp_types::SemanticTableCell::new(hwp_types::CellCoordinate { row: 0, col: 0 })
            }],
        };

        // Act
        let result = semantic_table_to_structured(&semantic_table);

        // Assert
        assert_eq!(result.rows[0][0].blocks.len(), 1);
        match &result.rows[0][0].blocks[0] {
            CellBlock::Paragraph(p) => assert_eq!(p.plain_text(), "Content"),
            _ => panic!("Expected a paragraph block"),
        }
    }
}
