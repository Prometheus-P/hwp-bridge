// crates/hwp-types/src/tags.rs
//! HWP 레코드 태그 상수
//!
//! HWP 파일의 DocInfo 및 BodyText 스트림에서 사용되는 레코드 태그 ID입니다.

// === DocInfo Tags (0x00 - 0x1F) ===

/// 문서 속성
pub const DOCUMENT_PROPERTIES: u16 = 0x00;
/// ID 매핑 테이블
pub const ID_MAPPINGS: u16 = 0x01;
/// 바이너리 데이터
pub const BIN_DATA: u16 = 0x02;
/// 글꼴 이름
pub const FACE_NAME: u16 = 0x03;
/// 테두리/배경
pub const BORDER_FILL: u16 = 0x04;
/// 글자 모양
pub const CHAR_SHAPE: u16 = 0x07;
/// 탭 정의
pub const TAB_DEF: u16 = 0x08;
/// 문단 모양
pub const PARA_SHAPE: u16 = 0x09;
/// 스타일
pub const STYLE: u16 = 0x0A;
/// 메모 모양
pub const MEMO_SHAPE: u16 = 0x0B;

// === BodyText Tags (0x40 - 0x7F) ===

/// 문단 헤더
pub const PARA_HEADER: u16 = 0x42;
/// 문단 텍스트
pub const PARA_TEXT: u16 = 0x43;
/// 문단 글자 모양
pub const PARA_CHAR_SHAPE: u16 = 0x44;
/// 문단 라인 세그먼트
pub const PARA_LINE_SEG: u16 = 0x45;
/// 문단 범위 태그
pub const PARA_RANGE_TAG: u16 = 0x46;
/// 컨트롤 헤더
pub const CTRL_HEADER: u16 = 0x47;

// === Table Tags ===

/// 표
pub const TABLE: u16 = 0x4D;
/// 표 셀
pub const TABLE_CELL: u16 = 0x4E;

// === Shape Tags ===

/// 도형 컴포넌트
pub const SHAPE_COMPONENT: u16 = 0x51;
/// 선
pub const SHAPE_COMPONENT_LINE: u16 = 0x52;
/// 사각형
pub const SHAPE_COMPONENT_RECTANGLE: u16 = 0x53;
/// 타원
pub const SHAPE_COMPONENT_ELLIPSE: u16 = 0x54;
/// 그림
pub const SHAPE_COMPONENT_PICTURE: u16 = 0x57;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_correct_value_when_para_header_referenced() {
        // Arrange & Act
        let tag = PARA_HEADER;

        // Assert
        assert_eq!(tag, 0x42);
    }

    #[test]
    fn test_should_return_correct_value_when_char_shape_referenced() {
        // Arrange & Act
        let tag = CHAR_SHAPE;

        // Assert
        assert_eq!(tag, 0x07);
    }

    #[test]
    fn test_should_match_tag_id_when_compared() {
        // Arrange
        let parsed_tag_id: u16 = 0x42;

        // Act & Assert
        assert_eq!(parsed_tag_id, PARA_HEADER);
    }

    #[test]
    fn test_should_distinguish_docinfo_from_bodytext_tags() {
        // Arrange & Act & Assert
        // DocInfo tags are in 0x00-0x1F range
        assert!(DOCUMENT_PROPERTIES < 0x40);
        assert!(CHAR_SHAPE < 0x40);
        assert!(PARA_SHAPE < 0x40);

        // BodyText tags are in 0x40-0x7F range
        assert!(PARA_HEADER >= 0x40);
        assert!(PARA_TEXT >= 0x40);
        assert!(TABLE >= 0x40);
    }
}
