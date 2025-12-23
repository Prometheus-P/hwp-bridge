// crates/hwp-types/src/structured.rs

//! MCP/LLM용 구조화 문서 타입
//!
//! LLM이 문서 구조를 이해할 수 있도록 설계된 타입입니다.
//! JSON 직렬화 시 가독성과 의미 전달에 최적화되어 있습니다.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

fn default_schema_version() -> String {
    "1".to_string()
}

// ═══════════════════════════════════════════════════════════════════════════
// 최상위 문서 구조
// ═══════════════════════════════════════════════════════════════════════════

/// LLM-친화적 구조화 문서 (MCP export용)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredDocument {
    /// 문서 메타데이터
    pub metadata: StructuredMetadata,
    /// 문서 개요 (헤딩 기반 트리)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub outline: Vec<OutlineItem>,
    /// 전체 섹션
    pub sections: Vec<StructuredSection>,
    /// 스타일 정의 (참조용, 옵션)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<StyleDefinitions>,
}

impl StructuredDocument {
    /// 새 빈 문서 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 섹션 추가
    pub fn add_section(&mut self, section: StructuredSection) {
        self.sections.push(section);
    }

    /// 전체 텍스트 추출 (plain text)
    pub fn extract_text(&self) -> String {
        self.sections
            .iter()
            .flat_map(|s| s.content.iter())
            .filter_map(|block| match block {
                ContentBlock::Paragraph(p) => Some(p.plain_text()),
                ContentBlock::Table(t) => Some(t.to_text()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 총 문단 수
    pub fn paragraph_count(&self) -> usize {
        self.sections
            .iter()
            .flat_map(|s| s.content.iter())
            .filter(|b| matches!(b, ContentBlock::Paragraph(_)))
            .count()
    }

    /// 총 표 수
    pub fn table_count(&self) -> usize {
        self.sections
            .iter()
            .flat_map(|s| s.content.iter())
            .filter(|b| matches!(b, ContentBlock::Table(_)))
            .count()
    }
}

/// 문서 메타데이터
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredMetadata {
    /// JSON schema version (breaking changes bump major)
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    /// HWP 버전
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hwp_version: Option<String>,
    /// 총 페이지 수 (추정)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<u32>,
    /// 총 글자 수
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_count: Option<u32>,
    /// 암호화 여부
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_encrypted: bool,
    /// 배포용 문서 여부
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_distribution: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// 문서 개요
// ═══════════════════════════════════════════════════════════════════════════

/// 문서 개요 항목 (헤딩 기반)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineItem {
    /// 제목 텍스트
    pub title: String,
    /// 헤딩 레벨 (1-6)
    pub level: u8,
    /// 섹션/문단 위치 참조
    pub location: ContentLocation,
    /// 하위 항목
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<OutlineItem>,
}

/// 콘텐츠 위치 참조
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContentLocation {
    pub section_index: usize,
    pub block_index: usize,
}

// ═══════════════════════════════════════════════════════════════════════════
// 섹션
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 섹션
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredSection {
    /// 섹션 인덱스
    pub index: usize,
    /// 페이지 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_setup: Option<PageSetup>,
    /// 콘텐츠 블록 목록
    pub content: Vec<ContentBlock>,
}

impl StructuredSection {
    /// 새 섹션 생성
    pub fn new(index: usize) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }

    /// 콘텐츠 블록 추가
    pub fn add_content(&mut self, block: ContentBlock) {
        self.content.push(block);
    }

    /// 문단 추가 (편의 메서드)
    pub fn add_paragraph(&mut self, paragraph: StructuredParagraph) {
        self.content.push(ContentBlock::Paragraph(paragraph));
    }

    /// 표 추가 (편의 메서드)
    pub fn add_table(&mut self, table: StructuredTable) {
        self.content.push(ContentBlock::Table(table));
    }
}

/// 페이지 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSetup {
    /// 페이지 너비 (mm)
    pub width_mm: f32,
    /// 페이지 높이 (mm)
    pub height_mm: f32,
    /// 위쪽 여백 (mm)
    pub margin_top_mm: f32,
    /// 아래쪽 여백 (mm)
    pub margin_bottom_mm: f32,
    /// 왼쪽 여백 (mm)
    pub margin_left_mm: f32,
    /// 오른쪽 여백 (mm)
    pub margin_right_mm: f32,
    /// 페이지 방향
    pub orientation: PageOrientation,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self {
            width_mm: 210.0,  // A4
            height_mm: 297.0, // A4
            margin_top_mm: 20.0,
            margin_bottom_mm: 20.0,
            margin_left_mm: 20.0,
            margin_right_mm: 20.0,
            orientation: PageOrientation::Portrait,
        }
    }
}

/// 페이지 방향
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    #[default]
    Portrait,
    Landscape,
}

// ═══════════════════════════════════════════════════════════════════════════
// 콘텐츠 블록
// ═══════════════════════════════════════════════════════════════════════════

/// 콘텐츠 블록 (문단, 표, 이미지 등)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// 일반 문단
    Paragraph(StructuredParagraph),
    /// 표
    Table(StructuredTable),
    /// 차트
    Chart(StructuredChart),
    /// 이미지
    Image(StructuredImage),
    /// 수식
    Equation(StructuredEquation),
    /// 페이지 구분
    PageBreak,
    /// 각주
    Footnote(StructuredFootnote),
    /// 머리말
    Header(StructuredParagraph),
    /// 꼬리말
    Footer(StructuredParagraph),
}

// ═══════════════════════════════════════════════════════════════════════════
// 문단
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 문단
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredParagraph {
    /// 문단 유형
    #[serde(skip_serializing_if = "is_body_paragraph")]
    pub paragraph_type: ParagraphType,
    /// 텍스트 런 (서식 적용 단위)
    pub runs: Vec<TextRun>,
    /// 들여쓰기 레벨 (리스트용)
    #[serde(skip_serializing_if = "is_zero", default)]
    pub indent_level: u8,
    /// 정렬
    #[serde(skip_serializing_if = "is_left_alignment", default)]
    pub alignment: TextAlignment,
    /// 문단 앞 여백 (pt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_before: Option<f32>,
    /// 문단 뒤 여백 (pt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_after: Option<f32>,
}

fn is_body_paragraph(pt: &ParagraphType) -> bool {
    matches!(pt, ParagraphType::Body)
}

fn is_zero(n: &u8) -> bool {
    *n == 0
}

fn is_left_alignment(a: &TextAlignment) -> bool {
    matches!(a, TextAlignment::Left)
}

impl StructuredParagraph {
    /// 단순 텍스트로 문단 생성
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            runs: vec![TextRun::plain(text)],
            ..Default::default()
        }
    }

    /// 헤딩 문단 생성
    pub fn heading(level: u8, text: impl Into<String>) -> Self {
        Self {
            paragraph_type: ParagraphType::Heading { level },
            runs: vec![TextRun::plain(text)],
            ..Default::default()
        }
    }

    /// plain text 추출
    pub fn plain_text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }

    /// 텍스트 런 추가
    pub fn add_run(&mut self, run: TextRun) {
        self.runs.push(run);
    }

    /// 정렬 설정
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// 문단 유형
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParagraphType {
    /// 일반 본문
    #[default]
    Body,
    /// 제목 (레벨 1-6)
    Heading { level: u8 },
    /// 번호 매기기 목록
    NumberedList { number: String },
    /// 글머리 기호 목록
    BulletList { bullet: String },
    /// 인용문
    Quote,
    /// 코드 블록
    Code { language: Option<String> },
}

/// 텍스트 런 (동일 서식 적용 단위)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextRun {
    /// 텍스트 내용
    pub text: String,
    /// 서식 (None이면 기본 서식)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<InlineStyle>,
}

impl TextRun {
    /// 평범한 텍스트 런 생성
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: None,
        }
    }

    /// 스타일 있는 텍스트 런 생성
    pub fn styled(text: impl Into<String>, style: InlineStyle) -> Self {
        Self {
            text: text.into(),
            style: Some(style),
        }
    }

    /// 볼드 텍스트
    pub fn bold(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::bold())
    }

    /// 이탤릭 텍스트
    pub fn italic(text: impl Into<String>) -> Self {
        Self::styled(text, InlineStyle::italic())
    }
}

/// 인라인 서식
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InlineStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size_pt: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// 텍스트 색상 (hex: #RRGGBB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// 배경색 (hex: #RRGGBB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superscript: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscript: Option<bool>,
}

impl InlineStyle {
    pub fn bold() -> Self {
        Self {
            bold: Some(true),
            ..Default::default()
        }
    }

    pub fn italic() -> Self {
        Self {
            italic: Some(true),
            ..Default::default()
        }
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn with_font_size(mut self, pt: f32) -> Self {
        self.font_size_pt = Some(pt);
        self
    }
}

/// 텍스트 정렬
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
    Distribute,
}

// ═══════════════════════════════════════════════════════════════════════════
// 표
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 표
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredTable {
    /// 표 제목 (캡션)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// 행 수
    pub row_count: usize,
    /// 열 수
    pub col_count: usize,
    /// 헤더 행 수
    #[serde(skip_serializing_if = "is_zero_usize", default)]
    pub header_rows: usize,
    /// 셀 데이터 (row-major order)
    pub rows: Vec<Vec<StructuredTableCell>>,
    /// 열 너비 비율
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_widths: Option<Vec<f32>>,
    /// 병합 정보
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub merged_cells: Vec<TableMergeRegion>,
    /// 표의 격자 맵
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub grid: Vec<Vec<TableGridSlot>>,
}

fn is_zero_usize(n: &usize) -> bool {
    *n == 0
}

impl StructuredTable {
    /// 새 표 생성
    pub fn new(row_count: usize, col_count: usize) -> Self {
        Self {
            row_count,
            col_count,
            rows: Vec::with_capacity(row_count),
            ..Default::default()
        }
    }

    /// 행 추가
    pub fn add_row(&mut self, mut row: Vec<StructuredTableCell>) {
        let row_index = self.rows.len();
        for (col_index, cell) in row.iter_mut().enumerate() {
            if cell.position.row == usize::MAX && cell.position.col == usize::MAX {
                cell.position = CellCoordinate {
                    row: row_index,
                    col: col_index,
                };
            }
        }
        self.rows.push(row);
        self.rebuild_grid();
    }

    /// 셀 접근
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&StructuredTableCell> {
        self.rows.get(row).and_then(|r| r.get(col))
    }

    /// 격자 정보를 재구축합니다.
    pub fn rebuild_grid(&mut self) {
        if self.row_count == 0 || self.col_count == 0 {
            self.grid.clear();
            self.merged_cells.clear();
            return;
        }

        let mut grid = Vec::with_capacity(self.row_count);
        for r in 0..self.row_count {
            let mut row = Vec::with_capacity(self.col_count);
            for c in 0..self.col_count {
                row.push(TableGridSlot {
                    position: CellCoordinate { row: r, col: c },
                    anchor: CellCoordinate { row: r, col: c },
                    is_anchor: true,
                });
            }
            grid.push(row);
        }
        let mut merges = Vec::new();

        for row in &self.rows {
            for cell in row {
                let anchor = cell.position;
                let col_span = cell.col_span.max(1);
                let row_span = cell.row_span.max(1);

                if col_span > 1 || row_span > 1 {
                    merges.push(TableMergeRegion {
                        anchor,
                        col_span,
                        row_span,
                    });
                }

                let end_row = (anchor.row + row_span).min(self.row_count);
                let end_col = (anchor.col + col_span).min(self.col_count);
                for r in anchor.row..end_row {
                    for c in anchor.col..end_col {
                        if let Some(slot) = grid.get_mut(r).and_then(|rr| rr.get_mut(c)) {
                            slot.position = CellCoordinate { row: r, col: c };
                            slot.anchor = anchor;
                            slot.is_anchor = r == anchor.row && c == anchor.col;
                        }
                    }
                }
            }
        }

        self.grid = grid;
        self.merged_cells = merges;
    }

    /// 텍스트 표현으로 변환 (Markdown 스타일)
    pub fn to_text(&self) -> String {
        let mut result = String::new();

        if let Some(caption) = &self.caption {
            result.push_str(&format!("[표: {}]\n", caption));
        }

        for (i, row) in self.rows.iter().enumerate() {
            let row_text: Vec<String> = row
                .iter()
                .filter(|c| !c.hidden_by_span)
                .map(|c| {
                    let mut text = c.plain_text();
                    if c.col_span > 1 || c.row_span > 1 {
                        text.push_str(&format!(" [{}x{}]", c.col_span, c.row_span));
                    }
                    text
                })
                .collect();
            result.push_str(&format!("| {} |", row_text.join(" | ")));
            result.push('\n');

            // 헤더 구분선
            if i < self.header_rows && i == self.header_rows - 1 {
                let sep: Vec<&str> = vec!["---"; self.col_count];
                result.push_str(&format!("| {} |", sep.join(" | ")));
                result.push('\n');
            }
        }

        result
    }
}

/// 표 셀
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredTableCell {
    /// 셀 위치
    pub position: CellCoordinate,
    /// 셀 콘텐츠
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub blocks: Vec<CellBlock>,
    /// 열 병합
    #[serde(skip_serializing_if = "is_one", default = "default_one")]
    pub col_span: usize,
    /// 행 병합
    #[serde(skip_serializing_if = "is_one", default = "default_one")]
    pub row_span: usize,
    /// 헤더 셀 여부
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub is_header: bool,
    /// 병합으로 인해 표시되지 않는 셀
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub hidden_by_span: bool,
}

fn is_one(n: &usize) -> bool {
    *n == 1
}

fn default_one() -> usize {
    1
}

impl StructuredTableCell {
    /// 텍스트로 셀 생성
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            position: CellCoordinate { row: 0, col: 0 },
            blocks: vec![CellBlock::Paragraph(StructuredParagraph::from_text(
                text.into(),
            ))],
            col_span: 1,
            row_span: 1,
            is_header: false,
            hidden_by_span: false,
        }
    }

    /// 헤더 셀로 설정
    pub fn as_header(mut self) -> Self {
        self.is_header = true;
        self
    }

    /// 병합 설정
    pub fn with_span(mut self, col_span: usize, row_span: usize) -> Self {
        self.col_span = col_span;
        self.row_span = row_span;
        self
    }

    /// plain text 추출
    pub fn plain_text(&self) -> String {
        let mut text = Vec::new();
        for block in &self.blocks {
            match block {
                CellBlock::Paragraph(p) => text.push(p.plain_text()),
                CellBlock::Table(t) => text.push(t.to_text()),
                CellBlock::RawText { text: raw } => text.push(raw.clone()),
            }
        }
        text.join("\n")
    }

    /// 위치 지정
    pub fn with_position(mut self, row: usize, col: usize) -> Self {
        self.position = CellCoordinate { row, col };
        self
    }

    /// 블록 추가
    pub fn push_block(&mut self, block: CellBlock) {
        self.blocks.push(block);
    }

    /// 중첩 표 얻기
    pub fn nested_tables(&self) -> Vec<&StructuredTable> {
        self.blocks
            .iter()
            .filter_map(|block| match block {
                CellBlock::Table(table) => Some(table.as_ref()),
                _ => None,
            })
            .collect()
    }
}

/// 셀 좌표
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CellCoordinate {
    pub row: usize,
    pub col: usize,
}

impl Default for CellCoordinate {
    fn default() -> Self {
        Self {
            row: usize::MAX,
            col: usize::MAX,
        }
    }
}

/// 병합 영역 정보
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TableMergeRegion {
    pub anchor: CellCoordinate,
    pub col_span: usize,
    pub row_span: usize,
}

/// 격자 슬롯
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TableGridSlot {
    pub position: CellCoordinate,
    pub anchor: CellCoordinate,
    pub is_anchor: bool,
}

/// 셀 콘텐츠 블록 (문단/표/Raw 텍스트)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "block_type", rename_all = "snake_case")]
pub enum CellBlock {
    Paragraph(StructuredParagraph),
    Table(Box<StructuredTable>),
    RawText { text: String },
}

// ═══════════════════════════════════════════════════════════════════════════
// 이미지
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 이미지
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredImage {
    /// 대체 텍스트
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    /// 캡션
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// 이미지 너비 (pt)
    pub width_pt: f32,
    /// 이미지 높이 (pt)
    pub height_pt: f32,
    /// BinData ID 참조
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_data_id: Option<u16>,
    /// Base64 인코딩 데이터 (옵션)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_base64: Option<String>,
    /// MIME 타입
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl StructuredImage {
    pub fn new(width_pt: f32, height_pt: f32) -> Self {
        Self {
            width_pt,
            height_pt,
            ..Default::default()
        }
    }

    pub fn with_alt_text(mut self, alt: impl Into<String>) -> Self {
        self.alt_text = Some(alt.into());
        self
    }
}

/// 구조화된 차트 (OLE 기반)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredChart {
    /// BinData ID (있는 경우)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_data_id: Option<u16>,
    /// 스트림 유형 (Contents / OOXML.ChartContents 등)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_type: Option<String>,
    /// 차트 제목
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 차트 유형
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chart_type: Option<String>,
    /// 차트 데이터 격자
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_grid: Option<StructuredTable>,
    /// 추가 설명
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// 수식
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 수식
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredEquation {
    /// LaTeX 표현
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latex: Option<String>,
    /// MathML 표현
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mathml: Option<String>,
    /// 텍스트 표현 (fallback)
    pub text: String,
}

// ═══════════════════════════════════════════════════════════════════════════
// 각주
// ═══════════════════════════════════════════════════════════════════════════

/// 구조화된 각주
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredFootnote {
    /// 각주 번호/기호
    pub marker: String,
    /// 각주 내용
    pub content: Vec<StructuredParagraph>,
}

// ═══════════════════════════════════════════════════════════════════════════
// 스타일 정의
// ═══════════════════════════════════════════════════════════════════════════

/// 스타일 정의 (참조용)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StyleDefinitions {
    pub paragraph_styles: Vec<NamedParagraphStyle>,
    pub character_styles: Vec<NamedCharacterStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedParagraphStyle {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_style: Option<String>,
    pub alignment: TextAlignment,
    pub indent_pt: f32,
    pub space_before_pt: f32,
    pub space_after_pt: f32,
    pub line_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedCharacterStyle {
    pub name: String,
    pub font_family: String,
    pub font_size_pt: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub bold: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub italic: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Semantic (zero-copy) Models
// ═══════════════════════════════════════════════════════════════════════════

/// nom 파서가 즉시 소비할 수 있는 문단 프래그먼트
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct SemanticParagraph<'a> {
    /// 문단 헤더 (PARA_HEADER)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<ParagraphHeader>,
    /// 문단 텍스트 (PARA_TEXT)
    #[serde(borrow)]
    pub text: Cow<'a, str>,
    /// 텍스트 구간 정보
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub spans: Vec<SemanticSpan<'a>>,
}

impl<'a> SemanticParagraph<'a> {
    pub fn new_text<T: Into<Cow<'a, str>>>(text: T) -> Self {
        Self {
            header: None,
            spans: Vec::new(),
            text: text.into(),
        }
    }

    pub fn with_header(header: ParagraphHeader) -> Self {
        Self {
            header: Some(header),
            text: Cow::Borrowed(""),
            spans: Vec::new(),
        }
    }
}

/// 문단 헤더 메타데이터
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParagraphHeader {
    pub control_mask: u32,
    pub para_shape_id: u16,
    pub style_id: u8,
    pub column_type: u8,
    pub char_shape_count: u16,
    pub range_tag_count: u16,
    pub line_align_count: u16,
    pub instance_id: u32,
}

/// 텍스트 런 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct SemanticSpan<'a> {
    pub start: usize,
    pub len: usize,
    #[serde(borrow)]
    pub text: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_shape_id: Option<u16>,
}

/// nom 기반 표(Semantic) 표현
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct SemanticTable<'a> {
    pub properties: u32,
    pub rows: u16,
    pub cols: u16,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cells: Vec<SemanticTableCell<'a>>,
}

impl<'a> SemanticTable<'a> {
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            properties: 0,
            rows,
            cols,
            cells: Vec::new(),
        }
    }

    pub fn push_cell(&mut self, cell: SemanticTableCell<'a>) {
        self.cells.push(cell);
    }
}

/// zero-copy 표 셀
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct SemanticTableCell<'a> {
    pub address: CellCoordinate,
    pub col_span: u16,
    pub row_span: u16,
    pub size: (u32, u32),
    #[serde(borrow)]
    pub field_name: Cow<'a, str>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub paragraphs: Vec<SemanticParagraph<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nested_tables: Vec<SemanticTable<'a>>,
}

impl<'a> SemanticTableCell<'a> {
    pub fn new(address: CellCoordinate) -> Self {
        Self {
            address,
            col_span: 1,
            row_span: 1,
            size: (0, 0),
            field_name: Cow::Borrowed(""),
            paragraphs: Vec::new(),
            nested_tables: Vec::new(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_structured_document() {
        // Arrange & Act
        let mut doc = StructuredDocument::new();
        doc.metadata.title = Some("테스트 문서".to_string());

        let mut section = StructuredSection::new(0);
        section.add_paragraph(StructuredParagraph::heading(1, "제목"));
        section.add_paragraph(StructuredParagraph::from_text("본문 내용입니다."));
        doc.add_section(section);

        // Assert
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.paragraph_count(), 2);
    }

    #[test]
    fn test_should_serialize_to_json() {
        // Arrange
        let mut doc = StructuredDocument::new();
        doc.metadata.title = Some("JSON 테스트".to_string());

        let mut section = StructuredSection::new(0);
        section.add_paragraph(StructuredParagraph::from_text("Hello"));
        doc.add_section(section);

        // Act
        let json = serde_json::to_string_pretty(&doc).unwrap();

        // Assert
        assert!(json.contains("JSON 테스트"));
        assert!(json.contains("Hello"));
    }

    #[test]
    fn test_should_extract_plain_text() {
        // Arrange
        let mut doc = StructuredDocument::new();
        let mut section = StructuredSection::new(0);
        section.add_paragraph(StructuredParagraph::from_text("첫 번째"));
        section.add_paragraph(StructuredParagraph::from_text("두 번째"));
        doc.add_section(section);

        // Act
        let text = doc.extract_text();

        // Assert
        assert!(text.contains("첫 번째"));
        assert!(text.contains("두 번째"));
    }

    #[test]
    fn test_should_create_table() {
        // Arrange
        let mut table = StructuredTable::new(2, 3);
        table.header_rows = 1;

        // Header row
        table.add_row(vec![
            StructuredTableCell::from_text("항목").as_header(),
            StructuredTableCell::from_text("2023년").as_header(),
            StructuredTableCell::from_text("2024년").as_header(),
        ]);

        // Data row
        table.add_row(vec![
            StructuredTableCell::from_text("매출"),
            StructuredTableCell::from_text("100억"),
            StructuredTableCell::from_text("120억"),
        ]);

        // Act
        let text = table.to_text();

        // Assert
        assert!(text.contains("항목"));
        assert!(text.contains("---")); // header separator
        assert!(text.contains("120억"));
    }

    #[test]
    fn test_should_create_styled_text_run() {
        // Arrange & Act
        let run = TextRun::styled("강조 텍스트", InlineStyle::bold().with_color("#FF0000"));

        // Assert
        assert_eq!(run.text, "강조 텍스트");
        assert!(run.style.as_ref().unwrap().bold.unwrap());
        assert_eq!(
            run.style.as_ref().unwrap().color.as_ref().unwrap(),
            "#FF0000"
        );
    }

    #[test]
    fn test_should_roundtrip_json() {
        // Arrange
        let mut doc = StructuredDocument::new();
        doc.metadata.title = Some("Roundtrip Test".to_string());
        doc.metadata.hwp_version = Some("5.1.0.0".to_string());

        let mut section = StructuredSection::new(0);
        let mut para = StructuredParagraph::heading(1, "제목");
        para.runs.push(TextRun::bold(" (중요)"));
        section.add_paragraph(para);
        doc.add_section(section);

        // Act
        let json = serde_json::to_string(&doc).unwrap();
        let restored: StructuredDocument = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(doc.metadata.title, restored.metadata.title);
        assert_eq!(doc.sections.len(), restored.sections.len());
        assert_eq!(
            doc.sections[0].content.len(),
            restored.sections[0].content.len()
        );
    }
}
