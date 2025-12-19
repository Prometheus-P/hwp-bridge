// crates/hwp-core/src/converter/structured.rs

//! HwpDocument → StructuredDocument 변환
//!
//! 파싱된 HWP 문서를 LLM 친화적인 구조화 문서로 변환합니다.

use hwp_types::{
    Alignment, CellBlock, CharShape, ContentBlock, Control, FileHeader, HwpDocument, InlineStyle,
    ParaShape, Paragraph, ParagraphType, Section, StructuredDocument, StructuredMetadata,
    StructuredParagraph, StructuredSection, StructuredTable, StructuredTableCell, Table,
    TextAlignment, TextRun,
};
use std::fmt::Write;

/// HwpDocument를 StructuredDocument로 변환
pub fn to_structured_document(
    doc: &HwpDocument,
    header: Option<&FileHeader>,
) -> StructuredDocument {
    let mut structured = StructuredDocument::new();

    // 메타데이터 변환
    structured.metadata = convert_metadata(&doc.metadata, header);

    // 섹션 변환
    for (idx, section) in doc.sections.iter().enumerate() {
        let structured_section = convert_section(section, idx, &doc.char_shapes, &doc.para_shapes);
        structured.sections.push(structured_section);
    }

    // 문서 통계 업데이트
    update_statistics(&mut structured);

    structured
}

/// 메타데이터 변환
fn convert_metadata(
    metadata: &hwp_types::DocumentMetadata,
    header: Option<&FileHeader>,
) -> StructuredMetadata {
    let mut sm = StructuredMetadata::default();

    if !metadata.title.is_empty() {
        sm.title = Some(metadata.title.clone());
    }
    if !metadata.author.is_empty() {
        sm.author = Some(metadata.author.clone());
    }
    if !metadata.created_at.is_empty() {
        sm.created_at = Some(metadata.created_at.clone());
    }

    sm.is_encrypted = metadata.is_encrypted;
    sm.is_distribution = metadata.is_distribution;

    if let Some(h) = header {
        sm.hwp_version = Some(h.version.to_string());
    }

    sm
}

/// 섹션 변환
fn convert_section(
    section: &Section,
    index: usize,
    char_shapes: &[CharShape],
    para_shapes: &[ParaShape],
) -> StructuredSection {
    let mut ss = StructuredSection::new(index);

    for para in &section.paragraphs {
        let content_block = convert_paragraph(para, char_shapes, para_shapes);
        ss.content.push(content_block);
    }

    ss
}

/// 문단 변환
fn convert_paragraph(
    para: &Paragraph,
    char_shapes: &[CharShape],
    para_shapes: &[ParaShape],
) -> ContentBlock {
    let mut sp = StructuredParagraph::default();

    // 텍스트 런 생성
    if para.char_shapes.is_empty() {
        // 단일 런
        sp.runs.push(TextRun::plain(&para.text));
    } else {
        // 여러 런으로 분할
        let runs = split_text_by_char_shapes(&para.text, &para.char_shapes, char_shapes);
        sp.runs = runs;
    }

    // 문단 스타일 적용
    if let Some(ps) = para_shapes.get(para.para_shape_id as usize) {
        sp.alignment = convert_alignment(ps.attr.alignment());

        // 들여쓰기가 있으면 indent_level 설정
        if ps.indent > 0 {
            sp.indent_level = (ps.indent / 400) as u8; // 대략적인 레벨 계산
        }

        // 여백
        if ps.margin_top > 0 {
            sp.space_before = Some(hwpunit_to_pt(ps.margin_top));
        }
        if ps.margin_bottom > 0 {
            sp.space_after = Some(hwpunit_to_pt(ps.margin_bottom));
        }
    }

    // 문단 유형 감지 (헤딩 등)
    sp.paragraph_type =
        detect_paragraph_type(&sp.runs, para_shapes.get(para.para_shape_id as usize));

    ContentBlock::Paragraph(sp)
}

/// 텍스트를 CharShape 경계에서 분할
fn split_text_by_char_shapes(
    text: &str,
    char_shape_refs: &[(u32, u16)],
    char_shapes: &[CharShape],
) -> Vec<TextRun> {
    if char_shape_refs.is_empty() {
        return vec![TextRun::plain(text)];
    }

    let chars: Vec<char> = text.chars().collect();
    let mut runs = Vec::new();
    let mut sorted_refs = char_shape_refs.to_vec();
    sorted_refs.sort_by_key(|r| r.0);

    for i in 0..sorted_refs.len() {
        let (start_pos, shape_id) = sorted_refs[i];
        let end_pos = if i + 1 < sorted_refs.len() {
            sorted_refs[i + 1].0 as usize
        } else {
            chars.len()
        };

        let start = start_pos as usize;
        if start >= chars.len() {
            break;
        }
        let end = end_pos.min(chars.len());

        let run_text: String = chars[start..end].iter().collect();
        if run_text.is_empty() {
            continue;
        }

        let style = char_shapes.get(shape_id as usize).map(convert_char_shape);

        runs.push(TextRun {
            text: run_text,
            style,
        });
    }

    // 첫 번째 char_shape 이전의 텍스트 처리
    if let Some((first_pos, _)) = sorted_refs.first()
        && *first_pos > 0
    {
        let prefix: String = chars[..*first_pos as usize].iter().collect();
        if !prefix.is_empty() {
            runs.insert(0, TextRun::plain(prefix));
        }
    }

    if runs.is_empty() {
        runs.push(TextRun::plain(text));
    }

    runs
}

/// CharShape를 InlineStyle로 변환
fn convert_char_shape(cs: &CharShape) -> InlineStyle {
    let mut style = InlineStyle::default();

    if cs.attr.is_bold() {
        style.bold = Some(true);
    }
    if cs.attr.is_italic() {
        style.italic = Some(true);
    }
    if cs.attr.underline_type() > 0 {
        style.underline = Some(true);
    }
    if cs.attr.strikethrough_type() > 0 {
        style.strikethrough = Some(true);
    }
    if cs.attr.is_superscript() {
        style.superscript = Some(true);
    }
    if cs.attr.is_subscript() {
        style.subscript = Some(true);
    }

    // 폰트 크기 (1/100 pt → pt)
    if cs.base_size > 0 {
        style.font_size_pt = Some(cs.base_size as f32 / 100.0);
    }

    // 색상 (COLORREF → hex, 검정색(0x000000)은 기본값이므로 제외)
    if cs.text_color != 0 {
        style.color = Some(colorref_to_hex(cs.text_color));
    }

    // 배경색
    if cs.shade_color != 0 && cs.shade_color != 0xFFFFFF {
        style.background_color = Some(colorref_to_hex(cs.shade_color));
    }

    style
}

/// Alignment 변환
fn convert_alignment(align: Alignment) -> TextAlignment {
    match align {
        Alignment::Left => TextAlignment::Left,
        Alignment::Center => TextAlignment::Center,
        Alignment::Right => TextAlignment::Right,
        Alignment::Justify => TextAlignment::Justify,
        Alignment::Distribute => TextAlignment::Distribute,
    }
}

/// 문단 유형 감지
fn detect_paragraph_type(runs: &[TextRun], para_shape: Option<&ParaShape>) -> ParagraphType {
    // 텍스트 기반 감지
    let full_text: String = runs.iter().map(|r| r.text.as_str()).collect();
    let trimmed = full_text.trim();

    // 숫자 목록 패턴 (1. 2. 등)
    if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_digit())
        && (rest.starts_with('.') || rest.starts_with(')'))
    {
        let num: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
        return ParagraphType::NumberedList {
            number: format!("{}.", num),
        };
    }

    // 글머리 기호 목록
    let bullet_chars = ['•', '·', '-', '–', '—', '○', '●', '■', '□', '▪', '▫'];
    if let Some(first_char) = trimmed.chars().next()
        && bullet_chars.contains(&first_char)
    {
        return ParagraphType::BulletList {
            bullet: first_char.to_string(),
        };
    }

    // 폰트 크기 기반 헤딩 감지
    if let Some(run) = runs.first()
        && let Some(style) = &run.style
    {
        if let Some(size) = style.font_size_pt {
            if size >= 18.0 {
                return ParagraphType::Heading { level: 1 };
            } else if size >= 16.0 {
                return ParagraphType::Heading { level: 2 };
            } else if size >= 14.0 {
                return ParagraphType::Heading { level: 3 };
            }
        }
        // 볼드 + 중앙 정렬 = 제목 가능성
        if style.bold == Some(true)
            && let Some(ps) = para_shape
            && ps.attr.alignment() == Alignment::Center
        {
            return ParagraphType::Heading { level: 2 };
        }
    }

    ParagraphType::Body
}

/// Table을 StructuredTable로 변환
pub fn convert_table(table: &Table) -> StructuredTable {
    let mut st = StructuredTable::new(table.rows as usize, table.cols as usize);
    if table.rows > 0 {
        st.header_rows = 1;
    }

    let mut rows: Vec<Vec<StructuredTableCell>> = vec![Vec::new(); table.rows as usize];

    for cell in &table.cells {
        let row_idx = cell.row as usize;
        let col_idx = cell.col as usize;
        if row_idx >= rows.len() {
            continue;
        }

        let mut structured_cell = if cell.text.is_empty() {
            StructuredTableCell {
                blocks: Vec::new(),
                ..StructuredTableCell::from_text("")
            }
        } else {
            StructuredTableCell::from_text(cell.text.clone())
        };

        structured_cell.col_span = cell.col_span as usize;
        structured_cell.row_span = cell.row_span as usize;
        structured_cell.is_header = row_idx < st.header_rows;
        structured_cell = structured_cell.with_position(row_idx, col_idx);

        if structured_cell.blocks.is_empty() {
            structured_cell.push_block(CellBlock::RawText {
                text: format!("cell({},{})", row_idx + 1, col_idx + 1),
            });
        }

        rows[row_idx].push(structured_cell);
    }

    for row in rows {
        st.add_row(row);
    }

    st
}

/// Control을 ContentBlock으로 변환
pub fn convert_control(control: &Control) -> Option<ContentBlock> {
    match control {
        Control::Table(table) => Some(ContentBlock::Table(convert_table(table))),
        Control::Picture(pic) => {
            let img = hwp_types::StructuredImage {
                bin_data_id: Some(pic.bin_data_id),
                width_pt: hwpunit_to_pt(pic.width as i32),
                height_pt: hwpunit_to_pt(pic.height as i32),
                ..Default::default()
            };
            Some(ContentBlock::Image(img))
        }
        Control::Unknown { .. } => None,
    }
}

/// HWPUNIT (1/7200 inch) → pt (1/72 inch)
fn hwpunit_to_pt(hwpunit: i32) -> f32 {
    hwpunit as f32 / 100.0
}

/// COLORREF (0x00BBGGRR) → hex string (#RRGGBB)
fn colorref_to_hex(colorref: u32) -> String {
    let r = colorref & 0xFF;
    let g = (colorref >> 8) & 0xFF;
    let b = (colorref >> 16) & 0xFF;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// 문서 통계 업데이트
fn update_statistics(doc: &mut StructuredDocument) {
    let text = doc.extract_text();
    doc.metadata.char_count = Some(text.chars().count() as u32);
}

/// StructuredDocument → Semantic Markdown
pub fn to_semantic_markdown(doc: &StructuredDocument) -> String {
    let mut output = String::new();
    let mut table_counter = 0usize;

    if let Some(title) = &doc.metadata.title {
        let _ = writeln!(output, "# {}", title.trim());
        output.push('\n');
    }

    for (section_idx, section) in doc.sections.iter().enumerate() {
        let _ = writeln!(output, "## Section {}", section_idx + 1);
        output.push('\n');

        for block in &section.content {
            match block {
                ContentBlock::Paragraph(p) => {
                    output.push_str(&render_paragraph_markdown(p));
                }
                ContentBlock::Table(t) => {
                    render_table_markdown(
                        t,
                        &mut output,
                        1,
                        &mut table_counter,
                        &format!("sec{}-root", section_idx + 1),
                    );
                }
                ContentBlock::Image(img) => {
                    let alt = img.alt_text.clone().unwrap_or_default();
                    let _ = writeln!(output, "![{}](bin:{:?})\n", alt, img.bin_data_id);
                }
                ContentBlock::Equation(eq) => {
                    if let Some(latex) = &eq.latex {
                        let _ = writeln!(output, "$${}$$\n", latex);
                    } else {
                        let _ = writeln!(output, "`{}`\n", eq.text);
                    }
                }
                _ => {}
            }
        }
    }

    output
}

fn render_paragraph_markdown(paragraph: &StructuredParagraph) -> String {
    let text = paragraph.plain_text();
    if text.trim().is_empty() {
        return String::new();
    }

    match &paragraph.paragraph_type {
        ParagraphType::Heading { level } => {
            let hashes = "#".repeat((*level).clamp(1, 6) as usize);
            format!("{hashes} {}\n\n", text.trim())
        }
        ParagraphType::BulletList { .. } => format!("- {}\n", text.trim()),
        ParagraphType::NumberedList { number } => {
            format!("{} {}\n", number.trim(), text.trim())
        }
        ParagraphType::Quote => format!("> {}\n\n", text.trim()),
        ParagraphType::Code { .. } => format!("```\n{}\n```\n\n", text.trim_end()),
        _ => format!("{}\n\n", text.trim_end()),
    }
}

fn render_table_markdown(
    table: &StructuredTable,
    buf: &mut String,
    depth: usize,
    counter: &mut usize,
    anchor_prefix: &str,
) {
    let anchor = format!("{}-tbl{}", anchor_prefix, *counter);
    *counter += 1;

    let _ = writeln!(
        buf,
        "```table depth={} anchor={} rows={} cols={}",
        depth, anchor, table.row_count, table.col_count
    );
    if let Some(caption) = &table.caption {
        let _ = writeln!(buf, "caption: {}", caption);
    }

    for (row_idx, row) in table.rows.iter().enumerate() {
        let cells: Vec<String> = row
            .iter()
            .filter(|cell| !cell.hidden_by_span)
            .map(|cell| {
                let mut text = cell.plain_text();
                if text.trim().is_empty() {
                    if cell.position.row != usize::MAX && cell.position.col != usize::MAX {
                        text = format!("(r{}c{})", cell.position.row + 1, cell.position.col + 1);
                    }
                }
                if cell.col_span > 1 || cell.row_span > 1 {
                    text.push_str(&format!(" <span c={} r={}>", cell.col_span, cell.row_span));
                }
                if !cell.nested_tables().is_empty() {
                    text.push_str(&format!(" [nested:{}]", cell.nested_tables().len()));
                }
                text
            })
            .collect();

        let _ = writeln!(buf, "| {} |", cells.join(" | "));
        if row_idx < table.header_rows && row_idx == table.header_rows - 1 {
            let header_sep = vec!["---"; cells.len()];
            let _ = writeln!(buf, "| {} |", header_sep.join(" | "));
        }
    }

    buf.push_str("```\n\n");

    for row in &table.rows {
        for cell in row {
            if cell.hidden_by_span {
                continue;
            }
            for nested in cell.nested_tables() {
                let nested_anchor = if cell.position.row != usize::MAX {
                    format!(
                        "{}-r{}c{}",
                        anchor,
                        cell.position.row + 1,
                        cell.position.col + 1
                    )
                } else {
                    format!("{anchor}-child")
                };
                render_table_markdown(nested, buf, depth + 1, counter, &nested_anchor);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hwp_types::CharShapeAttr;

    #[test]
    fn test_should_convert_empty_document() {
        // Arrange
        let doc = HwpDocument::new();

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        assert!(structured.sections.is_empty());
    }

    #[test]
    fn test_should_convert_simple_document() {
        // Arrange
        let mut doc = HwpDocument::new();
        let mut section = Section::new();
        section.push_paragraph(Paragraph::new("Hello, HWP!"));
        doc.add_section(section);

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        assert_eq!(structured.sections.len(), 1);
        assert_eq!(structured.sections[0].content.len(), 1);

        if let ContentBlock::Paragraph(p) = &structured.sections[0].content[0] {
            assert_eq!(p.plain_text(), "Hello, HWP!");
        } else {
            panic!("Expected paragraph");
        }
    }

    #[test]
    fn test_should_convert_styled_paragraph() {
        // Arrange
        let mut doc = HwpDocument::new();

        // Add char shape (bold)
        let cs = CharShape {
            attr: CharShapeAttr::from_bits(0b1), // bold
            base_size: 1200,                     // 12pt
            ..Default::default()
        };
        doc.char_shapes.push(cs);

        let mut section = Section::new();
        let mut para = Paragraph::new("Bold text");
        para.add_char_shape(0, 0);
        section.push_paragraph(para);
        doc.add_section(section);

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        if let ContentBlock::Paragraph(p) = &structured.sections[0].content[0] {
            assert!(!p.runs.is_empty());
            if let Some(style) = &p.runs[0].style {
                assert_eq!(style.bold, Some(true));
                assert_eq!(style.font_size_pt, Some(12.0));
            }
        }
    }

    #[test]
    fn test_should_detect_numbered_list() {
        // Arrange
        let mut doc = HwpDocument::new();
        let mut section = Section::new();
        section.push_paragraph(Paragraph::new("1. First item"));
        section.push_paragraph(Paragraph::new("2. Second item"));
        doc.add_section(section);

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        if let ContentBlock::Paragraph(p) = &structured.sections[0].content[0] {
            assert!(matches!(
                p.paragraph_type,
                ParagraphType::NumberedList { .. }
            ));
        }
    }

    #[test]
    fn test_should_detect_bullet_list() {
        // Arrange
        let mut doc = HwpDocument::new();
        let mut section = Section::new();
        section.push_paragraph(Paragraph::new("• First item"));
        section.push_paragraph(Paragraph::new("• Second item"));
        doc.add_section(section);

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        if let ContentBlock::Paragraph(p) = &structured.sections[0].content[0] {
            assert!(matches!(p.paragraph_type, ParagraphType::BulletList { .. }));
        }
    }

    #[test]
    fn test_should_convert_colorref_to_hex() {
        // Arrange & Act & Assert
        assert_eq!(colorref_to_hex(0x000000), "#000000"); // Black
        assert_eq!(colorref_to_hex(0x0000FF), "#FF0000"); // Red (BGR → RGB)
        assert_eq!(colorref_to_hex(0x00FF00), "#00FF00"); // Green
        assert_eq!(colorref_to_hex(0xFF0000), "#0000FF"); // Blue (BGR → RGB)
        assert_eq!(colorref_to_hex(0xFFFFFF), "#FFFFFF"); // White
    }

    #[test]
    fn test_should_convert_hwpunit_to_pt() {
        // Arrange & Act & Assert
        assert_eq!(hwpunit_to_pt(100), 1.0); // 100 hwpunit = 1pt
        assert_eq!(hwpunit_to_pt(1200), 12.0); // 1200 hwpunit = 12pt
        assert_eq!(hwpunit_to_pt(0), 0.0);
    }

    #[test]
    fn test_should_update_statistics() {
        // Arrange
        let mut doc = HwpDocument::new();
        let mut section = Section::new();
        section.push_paragraph(Paragraph::new("Hello"));
        doc.add_section(section);

        // Act
        let structured = to_structured_document(&doc, None);

        // Assert
        assert_eq!(structured.metadata.char_count, Some(5));
    }
}
