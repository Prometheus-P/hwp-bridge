// crates/hwp-types/src/record.rs
//! HWP 레코드 헤더 타입
//!
//! 모든 HWP 레코드는 공통 헤더로 시작합니다.

use serde::{Deserialize, Serialize};

/// HWP 레코드 헤더 (4바이트 또는 8바이트)
///
/// 모든 HWP 레코드는 이 헤더로 시작합니다.
/// - tag_id: 레코드 종류 (10 bits, 0-1023)
/// - level: 중첩 깊이 (10 bits)
/// - size: 데이터 크기 (12 bits 기본, 0xFFF인 경우 4바이트 확장)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordHeader {
    /// 태그 ID (10 bits, 0-1023)
    pub tag_id: u16,
    /// 레벨 (10 bits, 중첩 깊이)
    pub level: u16,
    /// 데이터 크기 (12 bits 기본, 확장 시 4바이트 추가)
    /// 0xFFF(4095)인 경우 뒤따르는 4바이트가 실제 크기
    pub size: u32,
}

impl RecordHeader {
    /// 새 RecordHeader 생성
    pub fn new(tag_id: u16, level: u16, size: u32) -> Self {
        Self {
            tag_id,
            level,
            size,
        }
    }

    /// 확장 크기 사용 여부 (size가 4095 이상인 경우)
    pub fn is_extended_size(&self) -> bool {
        self.size >= 0xFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_should_create_record_header_when_new_called() {
        // Arrange & Act
        let header = RecordHeader::new(0x42, 0, 100);

        // Assert
        assert_eq!(header.tag_id, 0x42);
        assert_eq!(header.level, 0);
        assert_eq!(header.size, 100);
    }

    #[test]
    fn test_should_create_default_header_when_default_called() {
        // Arrange & Act
        let header = RecordHeader::default();

        // Assert
        assert_eq!(header.tag_id, 0);
        assert_eq!(header.level, 0);
        assert_eq!(header.size, 0);
    }

    #[test]
    fn test_should_return_false_when_size_less_than_4095() {
        // Arrange
        let header = RecordHeader::new(0x42, 0, 100);

        // Act & Assert
        assert!(!header.is_extended_size());
    }

    #[test]
    fn test_should_return_true_when_size_equals_4095() {
        // Arrange
        let header = RecordHeader::new(0x42, 0, 0xFFF);

        // Act & Assert
        assert!(header.is_extended_size());
    }

    #[test]
    fn test_should_return_true_when_size_exceeds_4095() {
        // Arrange
        let header = RecordHeader::new(0x42, 0, 10000);

        // Act & Assert
        assert!(header.is_extended_size());
    }

    #[test]
    fn test_should_compare_equal_when_same_values() {
        // Arrange
        let header1 = RecordHeader::new(0x42, 1, 100);
        let header2 = RecordHeader::new(0x42, 1, 100);

        // Assert
        assert_eq!(header1, header2);
    }

    #[test]
    fn test_should_serialize_to_json_when_serde_used() {
        // Arrange
        let header = RecordHeader::new(0x42, 0, 100);

        // Act
        let json = serde_json::to_string(&header).unwrap();
        let deserialized: RecordHeader = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(header, deserialized);
    }
}
