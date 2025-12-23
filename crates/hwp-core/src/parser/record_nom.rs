// crates/hwp-core/src/parser/record_nom.rs

//! nom 기반 HWP 레코드 파서
//!
//! HWP 문서의 레코드 헤더 및 데이터를 nom 파서 콤비네이터로 처리합니다.

use crate::parser::{bodytext::table::parse_table, section::extract_text_from_para_text};
use hwp_types::{
    CellCoordinate, HwpError, ParagraphHeader, SemanticParagraph, SemanticSpan, SemanticTable,
    SemanticTableCell, tags::RecordTag,
};
use nom::{
    IResult, Parser,
    bytes::complete::take,
    multi::many0,
    number::complete::{le_u8, le_u16, le_u32},
};
use std::borrow::Cow;

/// Extended size marker (size가 4095면 다음 4바이트에 실제 크기)
const EXTENDED_SIZE_MARKER: u32 = 0xFFF;

// ═══════════════════════════════════════════════════════════════════════════
// 레코드 헤더 파싱
// ═══════════════════════════════════════════════════════════════════════════

/// HWP 레코드 헤더 (nom 파싱 결과)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordHeaderNom {
    /// 태그 (enum 디스패치용)
    pub tag: RecordTag,
    /// 원본 태그 ID (u16)
    pub tag_id: u16,
    /// 레벨 (중첩 깊이)
    pub level: u16,
    /// 데이터 크기 (바이트)
    pub size: u32,
}

impl RecordHeaderNom {
    /// 헤더가 extended size를 사용하는지 확인
    pub fn is_extended(&self) -> bool {
        self.size >= EXTENDED_SIZE_MARKER
    }

    /// 전체 레코드 크기 (헤더 + 데이터)
    pub fn total_size(&self) -> usize {
        let header_size = if self.is_extended() { 8 } else { 4 };
        header_size + self.size as usize
    }
}

/// 레코드 헤더 파싱 (4바이트 기본, 필요시 8바이트)
///
/// HWP 레코드 헤더 비트 레이아웃:
/// | Size (12 bits) | Level (10 bits) | TagID (10 bits) |
/// |    31-20       |     19-10       |      9-0        |
pub fn parse_record_header(input: &[u8]) -> IResult<&[u8], RecordHeaderNom> {
    let (input, dword) = le_u32(input)?;

    // 비트 추출 (리틀 엔디안이므로 LSB부터)
    let tag_id = (dword & 0x3FF) as u16; // bits 0-9
    let level = ((dword >> 10) & 0x3FF) as u16; // bits 10-19
    let size_field = (dword >> 20) & 0xFFF; // bits 20-31

    // Extended size 처리
    if size_field == EXTENDED_SIZE_MARKER {
        let (input, extended_size) = le_u32(input)?;
        Ok((
            input,
            RecordHeaderNom {
                tag: RecordTag::from(tag_id),
                tag_id,
                level,
                size: extended_size,
            },
        ))
    } else {
        Ok((
            input,
            RecordHeaderNom {
                tag: RecordTag::from(tag_id),
                tag_id,
                level,
                size: size_field,
            },
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 레코드 전체 파싱
// ═══════════════════════════════════════════════════════════════════════════

/// 레코드 (헤더 + 데이터)
#[derive(Debug, Clone)]
pub struct RecordNom<'a> {
    pub header: RecordHeaderNom,
    pub data: &'a [u8],
}

impl<'a> RecordNom<'a> {
    /// 태그 반환
    pub fn tag(&self) -> RecordTag {
        self.header.tag
    }

    /// 레벨 반환
    pub fn level(&self) -> u16 {
        self.header.level
    }

    /// 데이터 크기 반환
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// 데이터를 소유권 있는 Vec로 복사
    pub fn data_to_vec(&self) -> Vec<u8> {
        self.data.to_vec()
    }
}

/// 레코드 전체 파싱 (헤더 + 데이터)
pub fn parse_record(input: &[u8]) -> IResult<&[u8], RecordNom<'_>> {
    let (input, header) = parse_record_header(input)?;
    let (input, data) = take(header.size as usize)(input)?;

    Ok((input, RecordNom { header, data }))
}

/// 여러 레코드를 연속으로 파싱
pub fn parse_records(input: &[u8]) -> IResult<&[u8], Vec<RecordNom<'_>>> {
    many0(parse_record).parse(input)
}

// ═══════════════════════════════════════════════════════════════════════════
// 스트리밍 이터레이터
// ═══════════════════════════════════════════════════════════════════════════

/// 레코드 스트림 이터레이터 (zero-copy)
pub struct RecordIteratorNom<'a> {
    data: &'a [u8],
}

impl<'a> RecordIteratorNom<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// 남은 바이트 수
    pub fn remaining(&self) -> usize {
        self.data.len()
    }
}

impl<'a> Iterator for RecordIteratorNom<'a> {
    type Item = Result<RecordNom<'a>, HwpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        match parse_record(self.data) {
            Ok((remaining, record)) => {
                self.data = remaining;
                Some(Ok(record))
            }
            Err(e) => Some(Err(HwpError::ParseError(format!(
                "Record parsing failed: {:?}",
                e
            )))),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 태그 필터링 이터레이터
// ═══════════════════════════════════════════════════════════════════════════

/// 특정 태그만 필터링하는 이터레이터
pub struct FilteredRecordIterator<'a> {
    inner: RecordIteratorNom<'a>,
    filter_tags: Vec<RecordTag>,
}

impl<'a> FilteredRecordIterator<'a> {
    pub fn new(data: &'a [u8], filter_tags: Vec<RecordTag>) -> Self {
        Self {
            inner: RecordIteratorNom::new(data),
            filter_tags,
        }
    }
}

impl<'a> Iterator for FilteredRecordIterator<'a> {
    type Item = Result<RecordNom<'a>, HwpError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(record) => {
                    if self.filter_tags.contains(&record.header.tag) {
                        return Some(Ok(record));
                    }
                    // 필터에 없으면 스킵
                }
                Err(e) => return Some(Err(e)),
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 모델 변환
// ═══════════════════════════════════════════════════════════════════════════

impl<'a> TryFrom<&RecordNom<'a>> for SemanticParagraph<'a> {
    type Error = HwpError;

    fn try_from(record: &RecordNom<'a>) -> Result<Self, Self::Error> {
        match record.tag() {
            RecordTag::ParaHeader => {
                let (_, header) = parse_para_header_nom(record.data).map_err(|_| {
                    HwpError::ParseError("Failed to parse PARA_HEADER payload".to_string())
                })?;
                Ok(SemanticParagraph::with_header(header))
            }
            RecordTag::ParaText => {
                let text = extract_text_from_para_text(record.data)?;
                let span = SemanticSpan {
                    start: 0,
                    len: text.chars().count(),
                    text: Cow::Owned(text.clone()),
                    char_shape_id: None,
                };
                Ok(SemanticParagraph {
                    header: None,
                    spans: vec![span],
                    text: Cow::Owned(text),
                })
            }
            _ => Err(HwpError::ParseError(format!(
                "Record {:?} is not a paragraph tag",
                record.tag()
            ))),
        }
    }
}

impl<'a> TryFrom<&RecordNom<'a>> for SemanticTable<'a> {
    type Error = HwpError;

    fn try_from(record: &RecordNom<'a>) -> Result<Self, Self::Error> {
        if record.tag() != RecordTag::Table {
            return Err(HwpError::ParseError(format!(
                "Record {:?} is not TABLE",
                record.tag()
            )));
        }

        let (_, parsed) = parse_table(record.data)
            .map_err(|_| HwpError::ParseError("Failed to parse TABLE record".into()))?;

        let mut semantic = SemanticTable::new(parsed.rows, parsed.cols);
        semantic.properties = parsed.properties;

        for cell in parsed.cells {
            let mut semantic_cell = SemanticTableCell::new(CellCoordinate {
                row: cell.row as usize,
                col: cell.col as usize,
            });
            semantic_cell.col_span = cell.col_span;
            semantic_cell.row_span = cell.row_span;
            semantic_cell.size = (cell.width, cell.height);
            semantic_cell.field_name = Cow::Owned(cell.field_name.clone());

            if !cell.text.is_empty() {
                semantic_cell
                    .paragraphs
                    .push(SemanticParagraph::new_text(cell.text.clone()));
            }

            semantic.push_cell(semantic_cell);
        }

        Ok(semantic)
    }
}

fn parse_para_header_nom(input: &[u8]) -> IResult<&[u8], ParagraphHeader> {
    let (input, control_mask) = le_u32(input)?;
    let (input, para_shape_id) = le_u16(input)?;
    let (input, style_id) = le_u8(input)?;
    let (input, column_type) = le_u8(input)?;
    let (input, char_shape_count) = le_u16(input)?;
    let (input, range_tag_count) = le_u16(input)?;
    let (input, line_align_count) = le_u16(input)?;
    let (input, instance_id) = le_u32(input)?;

    Ok((
        input,
        ParagraphHeader {
            control_mask,
            para_shape_id,
            style_id,
            column_type,
            char_shape_count,
            range_tag_count,
            line_align_count,
            instance_id,
        },
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// 유틸리티 함수
// ═══════════════════════════════════════════════════════════════════════════

/// 레코드 스트림에서 특정 태그의 레코드만 추출
pub fn extract_records_by_tag<'a>(
    data: &'a [u8],
    tag: RecordTag,
) -> Result<Vec<RecordNom<'a>>, HwpError> {
    RecordIteratorNom::new(data)
        .filter_map(|r| match r {
            Ok(record) if record.header.tag == tag => Some(Ok(record)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect()
}

/// 레코드 스트림에서 첫 번째 매칭 레코드 찾기
pub fn find_first_record<'a>(
    data: &'a [u8],
    tag: RecordTag,
) -> Result<Option<RecordNom<'a>>, HwpError> {
    for result in RecordIteratorNom::new(data) {
        let record = result?;
        if record.header.tag == tag {
            return Ok(Some(record));
        }
    }
    Ok(None)
}

/// 레코드 데이터에서 u16 파싱
pub fn parse_record_u16(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 <= data.len() {
        Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
    } else {
        None
    }
}

/// 레코드 데이터에서 u32 파싱
pub fn parse_record_u32(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 <= data.len() {
        Some(u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════════════

    /// 레코드 헤더 바이트 생성 (normal size)
    fn create_record_header(tag_id: u16, level: u16, size: u32) -> Vec<u8> {
        assert!(tag_id < 1024, "tag_id must be < 1024 (10 bits)");
        assert!(level < 1024, "level must be < 1024 (10 bits)");
        assert!(size < 4095, "size must be < 4095 for normal header");

        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
        dword.to_le_bytes().to_vec()
    }

    /// 레코드 헤더 바이트 생성 (extended size)
    fn create_extended_record_header(tag_id: u16, level: u16, size: u32) -> Vec<u8> {
        assert!(tag_id < 1024, "tag_id must be < 1024 (10 bits)");
        assert!(level < 1024, "level must be < 1024 (10 bits)");

        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (EXTENDED_SIZE_MARKER << 20);
        let mut bytes = dword.to_le_bytes().to_vec();
        bytes.extend_from_slice(&size.to_le_bytes());
        bytes
    }

    // ═══════════════════════════════════════════════════════════════
    // RecordHeader Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_header_with_nom_when_normal_size() {
        // Arrange: Tag=0x43 (PARA_TEXT), Level=0, Size=100
        let data = create_record_header(0x43, 0, 100);

        // Act
        let result = parse_record_header(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, header) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(header.tag, RecordTag::ParaText);
        assert_eq!(header.tag_id, 0x43);
        assert_eq!(header.level, 0);
        assert_eq!(header.size, 100);
    }

    #[test]
    fn test_should_parse_header_with_nom_when_extended_size() {
        // Arrange: Tag=0x43, Level=1, Size=10000 (extended)
        let data = create_extended_record_header(0x43, 1, 10000);

        // Act
        let result = parse_record_header(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, header) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(header.tag, RecordTag::ParaText);
        assert_eq!(header.level, 1);
        assert_eq!(header.size, 10000);
    }

    #[test]
    fn test_should_convert_tag_id_to_enum() {
        // Arrange
        let test_cases = [
            (0x10u16, RecordTag::DocumentProperties),
            (0x13, RecordTag::FaceName),
            (0x15, RecordTag::CharShape),
            (0x19, RecordTag::ParaShape),
            (0x42, RecordTag::ParaHeader),
            (0x43, RecordTag::ParaText),
            (0x4D, RecordTag::Table),
            (0xFF, RecordTag::Unknown(0xFF)),
        ];

        for (tag_id, expected) in test_cases {
            // Act
            let tag = RecordTag::from(tag_id);

            // Assert
            assert_eq!(tag, expected, "tag_id 0x{:02X}", tag_id);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // Record Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_record_with_data() {
        // Arrange: Header + 10 bytes of data
        let mut data = create_record_header(0x43, 0, 10);
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Act
        let result = parse_record(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, record) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(record.header.tag, RecordTag::ParaText);
        assert_eq!(record.data, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    // ═══════════════════════════════════════════════════════════════
    // Iterator Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_iterate_multiple_records_with_nom() {
        // Arrange: Two records
        let mut data = create_record_header(0x42, 0, 4); // PARA_HEADER
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        data.extend_from_slice(&create_record_header(0x43, 0, 2)); // PARA_TEXT
        data.extend_from_slice(&[0x11, 0x22]);

        // Act
        let records: Vec<_> = RecordIteratorNom::new(&data).collect();

        // Assert
        assert_eq!(records.len(), 2);
        assert!(records[0].is_ok());
        assert!(records[1].is_ok());

        let r1 = records[0].as_ref().unwrap();
        let r2 = records[1].as_ref().unwrap();

        assert_eq!(r1.tag(), RecordTag::ParaHeader);
        assert_eq!(r2.tag(), RecordTag::ParaText);
    }

    #[test]
    fn test_should_filter_records_by_tag() {
        // Arrange: Mix of different tags
        let mut data = Vec::new();

        // PARA_HEADER (0x42)
        data.extend_from_slice(&create_record_header(0x42, 0, 2));
        data.extend_from_slice(&[0x01, 0x02]);

        // PARA_TEXT (0x43)
        data.extend_from_slice(&create_record_header(0x43, 0, 4));
        data.extend_from_slice(&[0x41, 0x00, 0x42, 0x00]); // "AB" in UTF-16LE

        // PARA_CHAR_SHAPE (0x44)
        data.extend_from_slice(&create_record_header(0x44, 0, 2));
        data.extend_from_slice(&[0x03, 0x04]);

        // Another PARA_TEXT (0x43)
        data.extend_from_slice(&create_record_header(0x43, 0, 2));
        data.extend_from_slice(&[0x43, 0x00]); // "C" in UTF-16LE

        // Act
        let para_texts = extract_records_by_tag(&data, RecordTag::ParaText).unwrap();

        // Assert
        assert_eq!(para_texts.len(), 2);
        assert_eq!(para_texts[0].data_size(), 4);
        assert_eq!(para_texts[1].data_size(), 2);
    }

    #[test]
    fn test_should_find_first_record() {
        // Arrange
        let mut data = Vec::new();
        data.extend_from_slice(&create_record_header(0x42, 0, 2));
        data.extend_from_slice(&[0x01, 0x02]);
        data.extend_from_slice(&create_record_header(0x43, 0, 2));
        data.extend_from_slice(&[0x03, 0x04]);

        // Act
        let result = find_first_record(&data, RecordTag::ParaText).unwrap();

        // Assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().tag(), RecordTag::ParaText);
    }

    #[test]
    fn test_should_return_none_when_tag_not_found() {
        // Arrange
        let mut data = create_record_header(0x42, 0, 2);
        data.extend_from_slice(&[0x01, 0x02]);

        // Act
        let result = find_first_record(&data, RecordTag::Table).unwrap();

        // Assert
        assert!(result.is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // RecordTag Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_identify_docinfo_tags() {
        assert!(RecordTag::DocumentProperties.is_docinfo());
        assert!(RecordTag::CharShape.is_docinfo());
        assert!(RecordTag::ParaShape.is_docinfo());
        assert!(!RecordTag::ParaHeader.is_docinfo());
        assert!(!RecordTag::ParaText.is_docinfo());
    }

    #[test]
    fn test_should_identify_bodytext_tags() {
        assert!(RecordTag::ParaHeader.is_bodytext());
        assert!(RecordTag::ParaText.is_bodytext());
        assert!(RecordTag::Table.is_bodytext());
        assert!(!RecordTag::DocumentProperties.is_bodytext());
        assert!(!RecordTag::CharShape.is_bodytext());
    }

    #[test]
    fn test_should_display_tag_name() {
        assert_eq!(RecordTag::ParaText.name(), "PARA_TEXT");
        assert_eq!(RecordTag::Table.name(), "TABLE");
        assert_eq!(RecordTag::Unknown(0xFF).name(), "UNKNOWN");
    }
}
