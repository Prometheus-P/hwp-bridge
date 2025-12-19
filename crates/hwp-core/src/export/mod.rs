// crates/hwp-core/src/export/mod.rs
//!
//! High-level export helpers:
//! - HWP (OLE) -> StructuredDocument (JSON-friendly) and semantic markdown
//!
//! This is intentionally conservative: it focuses on deterministic output,
//! fail-fast on encrypted/distribution documents (handled by FileHeader::validate),
//! and resource limits (decompression/record count).

use std::io::{Read, Seek};

use crate::parser::{
    HwpOleFile, SectionLimits, decompress_section_with_limits, record_nom::RecordIteratorNom,
};
use hwp_types::{
    CellBlock, HwpError, ParagraphType, RecordTag, SemanticParagraph, SemanticTable,
    StructuredDocument, StructuredParagraph, StructuredSection, StructuredTable,
    StructuredTableCell,
};

/// Parse an HWP file into a StructuredDocument.
///
/// - Deterministic metadata: no timestamps.
/// - Title is provided by caller (recommended: file stem).
pub fn parse_structured_document<F: Read + Seek>(
    reader: F,
    title: Option<String>,
    limits: SectionLimits,
) -> Result<StructuredDocument, HwpError> {
    let mut ole = HwpOleFile::open(reader)?;
    let header = ole.header().clone();

    let mut doc = StructuredDocument::new();
    doc.metadata.title = title;
    doc.metadata.author = None;
    doc.metadata.created_at = None; // deterministic
    doc.metadata.is_encrypted = header.properties.is_encrypted();
    doc.metadata.is_distribution = header.properties.is_distribution();

    let is_compressed = header.properties.is_compressed();

    let mut section_idx = 0usize;
    loop {
        match ole.read_section(section_idx) {
            Ok(section_bytes) => {
                let decompressed = if is_compressed {
                    decompress_section_with_limits(&section_bytes, limits.max_decompressed_bytes)?
                } else {
                    if section_bytes.len() > limits.max_decompressed_bytes {
                        return Err(HwpError::SizeLimitExceeded(format!(
                            "Uncompressed section exceeds limit: {} > {} bytes",
                            section_bytes.len(),
                            limits.max_decompressed_bytes
                        )));
                    }
                    section_bytes
                };

                let mut structured_section = StructuredSection::new(section_idx);
                let mut record_count: usize = 0;

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
                        RecordTag::ParaText => {
                            let semantic = SemanticParagraph::try_from(&record)
                                .map_err(|e| HwpError::ParseError(e.to_string()))?;
                            if let Some(paragraph) = semantic_paragraph_to_structured(semantic) {
                                structured_section.add_paragraph(paragraph);
                            }
                        }
                        RecordTag::Table => {
                            let semantic = SemanticTable::try_from(&record)
                                .map_err(|e| HwpError::ParseError(e.to_string()))?;
                            let table = semantic_table_to_structured(&semantic);
                            structured_section.add_table(table);
                        }
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

fn semantic_table_to_structured(semantic: &SemanticTable<'_>) -> StructuredTable {
    let row_count = semantic.rows as usize;
    let col_count = semantic.cols as usize;

    let mut table = StructuredTable::new(row_count.max(1), col_count.max(1));
    table.header_rows = 0;

    // Initialize grid with default cells (position will be assigned by add_row)
    let mut grid: Vec<Vec<StructuredTableCell>> =
        vec![vec![StructuredTableCell::default(); col_count.max(1)]; row_count.max(1)];

    for cell in &semantic.cells {
        let r = (cell.address.row as usize).min(row_count.saturating_sub(1));
        let c = (cell.address.col as usize).min(col_count.saturating_sub(1));

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
            let text = paragraph.text.to_string();
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
