// crates/hwp-core/src/parser/record.rs

//! HWP 레코드 파싱
//!
//! HWP 문서의 DocInfo와 BodyText는 레코드 단위로 구성됩니다.
//! 각 레코드는 4바이트 헤더와 가변 길이 데이터로 이루어집니다.

use hwp_types::HwpError;

/// HWP 레코드 태그 ID
#[allow(dead_code)]
pub mod tags {
    // DocInfo tags (0x00 - 0x1F)
    pub const DOCUMENT_PROPERTIES: u16 = 0x00;
    pub const ID_MAPPINGS: u16 = 0x01;
    pub const BIN_DATA: u16 = 0x02;
    pub const FACE_NAME: u16 = 0x03;
    pub const BORDER_FILL: u16 = 0x04;
    pub const CHAR_SHAPE: u16 = 0x07;
    pub const TAB_DEF: u16 = 0x08;
    pub const PARA_SHAPE: u16 = 0x09;
    pub const STYLE: u16 = 0x0A;

    // BodyText tags (0x40 - 0x7F)
    pub const PARA_HEADER: u16 = 0x42;
    pub const PARA_TEXT: u16 = 0x43;
    pub const PARA_CHAR_SHAPE: u16 = 0x44;
    pub const PARA_LINE_SEG: u16 = 0x45;
    pub const PARA_RANGE_TAG: u16 = 0x46;
    pub const CTRL_HEADER: u16 = 0x47;

    // Table/Shape tags
    pub const TABLE: u16 = 0x4D;
    pub const LIST_HEADER: u16 = 0x4F;
    pub const PAGE_DEF: u16 = 0x50;
    pub const SHAPE_COMPONENT: u16 = 0x51;
}

/// Extended size marker (size가 4095면 다음 4바이트에 실제 크기)
const EXTENDED_SIZE_MARKER: u32 = 0xFFF;

/// HWP 레코드 헤더
///
/// 4바이트 구조:
/// - Bits 0-9: Tag ID (10 bits)
/// - Bits 10-19: Level (10 bits)
/// - Bits 20-31: Size (12 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordHeader {
    /// 태그 ID (레코드 종류)
    pub tag_id: u16,
    /// 레벨 (중첩 깊이)
    pub level: u16,
    /// 데이터 크기 (바이트)
    pub size: u32,
}

impl RecordHeader {
    /// 바이트 슬라이스에서 레코드 헤더 파싱
    ///
    /// # Returns
    /// (RecordHeader, consumed_bytes) 튜플
    pub fn parse(data: &[u8]) -> Result<(Self, usize), HwpError> {
        const HEADER_SIZE: usize = 4;

        if data.len() < HEADER_SIZE {
            return Err(HwpError::ParseError(format!(
                "Record header too short: need {} bytes, have {}",
                HEADER_SIZE,
                data.len()
            )));
        }

        // 4바이트 little-endian으로 읽기
        let dword = u32::from_le_bytes(data[0..4].try_into().unwrap());

        // 비트 추출
        let tag_id = (dword & 0x3FF) as u16;           // bits 0-9
        let level = ((dword >> 10) & 0x3FF) as u16;   // bits 10-19
        let size_field = (dword >> 20) & 0xFFF;       // bits 20-31

        // Extended size 처리
        if size_field == EXTENDED_SIZE_MARKER {
            const EXTENDED_HEADER_SIZE: usize = 8;

            if data.len() < EXTENDED_HEADER_SIZE {
                return Err(HwpError::ParseError(format!(
                    "Extended record header too short: need {} bytes, have {}",
                    EXTENDED_HEADER_SIZE,
                    data.len()
                )));
            }

            let size = u32::from_le_bytes(data[4..8].try_into().unwrap());

            Ok((Self { tag_id, level, size }, EXTENDED_HEADER_SIZE))
        } else {
            Ok((Self { tag_id, level, size: size_field }, HEADER_SIZE))
        }
    }
}

/// 레코드와 데이터를 함께 담는 구조체
#[derive(Debug, Clone)]
pub struct Record {
    pub header: RecordHeader,
    pub data: Vec<u8>,
}

impl Record {
    /// 바이트 슬라이스에서 레코드 파싱 (헤더 + 데이터)
    pub fn parse(data: &[u8]) -> Result<(Self, usize), HwpError> {
        let (header, header_size) = RecordHeader::parse(data)?;

        let data_start = header_size;
        let data_end = data_start + header.size as usize;

        if data.len() < data_end {
            return Err(HwpError::ParseError(format!(
                "Record data too short: need {} bytes, have {}",
                data_end,
                data.len()
            )));
        }

        let record_data = data[data_start..data_end].to_vec();

        Ok((
            Self {
                header,
                data: record_data,
            },
            data_end,
        ))
    }
}

/// 레코드 스트림을 순회하는 이터레이터
pub struct RecordIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> RecordIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }
}

impl<'a> Iterator for RecordIterator<'a> {
    type Item = Result<Record, HwpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        match Record::parse(&self.data[self.offset..]) {
            Ok((record, consumed)) => {
                self.offset += consumed;
                Some(Ok(record))
            }
            Err(e) => Some(Err(e)),
        }
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

        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | ((size as u32) << 20);
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
    fn test_should_parse_record_header_when_size_normal() {
        // Arrange: Tag=0x43 (PARA_TEXT), Level=0, Size=100
        let data = create_record_header(0x43, 0, 100);

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok(), "Should parse successfully");
        let (header, consumed) = result.unwrap();
        assert_eq!(header.tag_id, 0x43);
        assert_eq!(header.level, 0);
        assert_eq!(header.size, 100);
        assert_eq!(consumed, 4);
    }

    #[test]
    fn test_should_parse_record_header_when_size_extended() {
        // Arrange: Tag=0x43, Level=1, Size=10000 (extended)
        let data = create_extended_record_header(0x43, 1, 10000);

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok(), "Should parse extended header");
        let (header, consumed) = result.unwrap();
        assert_eq!(header.tag_id, 0x43);
        assert_eq!(header.level, 1);
        assert_eq!(header.size, 10000);
        assert_eq!(consumed, 8); // 4 + 4 for extended size
    }

    #[test]
    fn test_should_parse_max_normal_size() {
        // Arrange: Size=4094 (max normal size, not extended)
        let data = create_record_header(0x42, 2, 4094);

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok());
        let (header, _) = result.unwrap();
        assert_eq!(header.size, 4094);
    }

    #[test]
    fn test_should_parse_all_tag_bits() {
        // Arrange: Tag=0x3FF (max 10-bit value)
        let data = create_record_header(0x3FF, 0, 0);

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok());
        let (header, _) = result.unwrap();
        assert_eq!(header.tag_id, 0x3FF);
    }

    #[test]
    fn test_should_parse_all_level_bits() {
        // Arrange: Level=0x3FF (max 10-bit value)
        let data = create_record_header(0, 0x3FF, 0);

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_ok());
        let (header, _) = result.unwrap();
        assert_eq!(header.level, 0x3FF);
    }

    #[test]
    fn test_should_return_error_when_data_too_short() {
        // Arrange: Only 3 bytes (need 4)
        let data = vec![0x43, 0x00, 0x64];

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result, Err(HwpError::ParseError(_))));
    }

    #[test]
    fn test_should_return_error_when_extended_data_too_short() {
        // Arrange: Extended header but only 4 bytes (need 8)
        let dword: u32 = 0x43 | (EXTENDED_SIZE_MARKER << 20);
        let data = dword.to_le_bytes().to_vec();

        // Act
        let result = RecordHeader::parse(&data);

        // Assert
        assert!(result.is_err());
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
        let result = Record::parse(&data);

        // Assert
        assert!(result.is_ok());
        let (record, consumed) = result.unwrap();
        assert_eq!(record.header.tag_id, 0x43);
        assert_eq!(record.data.len(), 10);
        assert_eq!(record.data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(consumed, 14); // 4 header + 10 data
    }

    #[test]
    fn test_should_return_error_when_record_data_insufficient() {
        // Arrange: Header says 100 bytes but only 10 provided
        let mut data = create_record_header(0x43, 0, 100);
        data.extend_from_slice(&[0; 10]);

        // Act
        let result = Record::parse(&data);

        // Assert
        assert!(result.is_err());
    }

    // ═══════════════════════════════════════════════════════════════
    // RecordIterator Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_iterate_multiple_records() {
        // Arrange: Two records
        let mut data = create_record_header(0x42, 0, 4); // PARA_HEADER, 4 bytes
        data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        data.extend_from_slice(&create_record_header(0x43, 0, 2)); // PARA_TEXT, 2 bytes
        data.extend_from_slice(&[0x11, 0x22]);

        // Act
        let records: Vec<_> = RecordIterator::new(&data).collect();

        // Assert
        assert_eq!(records.len(), 2);
        assert!(records[0].is_ok());
        assert!(records[1].is_ok());

        let r1 = records[0].as_ref().unwrap();
        let r2 = records[1].as_ref().unwrap();

        assert_eq!(r1.header.tag_id, 0x42);
        assert_eq!(r2.header.tag_id, 0x43);
    }
}
