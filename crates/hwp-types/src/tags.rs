// crates/hwp-types/src/tags.rs
//! HWP 레코드 태그 정의
//!
//! HWP 파일의 DocInfo 및 BodyText 스트림에서 사용되는 레코드 태그 ID입니다.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// RecordTag Enum - 컴파일 타임 디스패치용
// ═══════════════════════════════════════════════════════════════════════════

/// HWP 레코드 태그 (컴파일 타임 디스패치)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecordTag {
    // === DocInfo Tags (0x00 - 0x1F) ===
    /// 문서 속성
    DocumentProperties,
    /// ID 매핑 테이블
    IdMappings,
    /// 바이너리 데이터
    BinData,
    /// 글꼴 이름
    FaceName,
    /// 테두리/배경
    BorderFill,
    /// 글자 모양
    CharShape,
    /// 탭 정의
    TabDef,
    /// 문단 모양
    ParaShape,
    /// 스타일
    Style,
    /// 메모 모양
    MemoShape,
    /// 문서 데이터
    DocData,
    /// 배포용 문서 데이터
    DistributeDocData,
    /// 호환 문서
    CompatibleDocument,
    /// 레이아웃 호환성
    LayoutCompatibility,
    /// 변경 추적 내용
    TrackChange,
    /// 변경 추적 작성자
    TrackChangeAuthor,

    // === BodyText Tags (0x40 - 0x7F) ===
    /// 문단 헤더
    ParaHeader,
    /// 문단 텍스트
    ParaText,
    /// 문단 글자 모양
    ParaCharShape,
    /// 문단 라인 세그먼트
    ParaLineSeg,
    /// 문단 범위 태그
    ParaRangeTag,
    /// 컨트롤 헤더
    CtrlHeader,
    /// 리스트 헤더
    ListHeader,
    /// 페이지 정의
    PageDef,
    /// 각주/미주 모양
    FootnoteShape,
    /// 페이지 테두리/배경
    PageBorderFill,

    // === Table Tags ===
    /// 표
    Table,
    /// 표 셀
    TableCell,

    // === Shape Tags ===
    /// 도형 컴포넌트
    ShapeComponent,
    /// 선
    ShapeComponentLine,
    /// 사각형
    ShapeComponentRectangle,
    /// 타원
    ShapeComponentEllipse,
    /// 호
    ShapeComponentArc,
    /// 다각형
    ShapeComponentPolygon,
    /// 곡선
    ShapeComponentCurve,
    /// 그림
    ShapeComponentPicture,
    /// OLE
    ShapeComponentOle,
    /// 컨테이너
    ShapeComponentContainer,

    // === Control Tags ===
    /// 필드 시작
    CtrlData,
    /// 양식 객체
    FormObject,
    /// 수식
    EqEdit,

    /// 알 수 없는 태그
    Unknown(u16),
}

impl RecordTag {
    /// 태그 ID를 u16으로 변환
    pub fn to_u16(self) -> u16 {
        match self {
            // DocInfo
            Self::DocumentProperties => 0x00,
            Self::IdMappings => 0x01,
            Self::BinData => 0x02,
            Self::FaceName => 0x03,
            Self::BorderFill => 0x04,
            Self::CharShape => 0x07,
            Self::TabDef => 0x08,
            Self::ParaShape => 0x09,
            Self::Style => 0x0A,
            Self::MemoShape => 0x0B,
            Self::DocData => 0x0C,
            Self::DistributeDocData => 0x0D,
            Self::CompatibleDocument => 0x12,
            Self::LayoutCompatibility => 0x13,
            Self::TrackChange => 0x14,
            Self::TrackChangeAuthor => 0x15,

            // BodyText
            Self::ParaHeader => 0x42,
            Self::ParaText => 0x43,
            Self::ParaCharShape => 0x44,
            Self::ParaLineSeg => 0x45,
            Self::ParaRangeTag => 0x46,
            Self::CtrlHeader => 0x47,
            Self::ListHeader => 0x4F,
            Self::PageDef => 0x50,
            Self::FootnoteShape => 0x51,
            Self::PageBorderFill => 0x52,

            // Table
            Self::Table => 0x4D,
            Self::TableCell => 0x4E,

            // Shape
            Self::ShapeComponent => 0x4B,
            Self::ShapeComponentLine => 0x52,
            Self::ShapeComponentRectangle => 0x53,
            Self::ShapeComponentEllipse => 0x54,
            Self::ShapeComponentArc => 0x55,
            Self::ShapeComponentPolygon => 0x56,
            Self::ShapeComponentCurve => 0x57,
            Self::ShapeComponentPicture => 0x58,
            Self::ShapeComponentOle => 0x59,
            Self::ShapeComponentContainer => 0x5A,

            // Control
            Self::CtrlData => 0x48,
            Self::FormObject => 0x60,
            Self::EqEdit => 0x61,

            Self::Unknown(id) => id,
        }
    }

    /// DocInfo 영역 태그인지 확인
    pub fn is_docinfo(&self) -> bool {
        self.to_u16() < 0x40
    }

    /// BodyText 영역 태그인지 확인
    pub fn is_bodytext(&self) -> bool {
        let id = self.to_u16();
        (0x40..0x80).contains(&id)
    }

    /// 컨트롤 태그인지 확인 (표, 이미지 등)
    pub fn is_control(&self) -> bool {
        matches!(
            self,
            Self::Table
                | Self::TableCell
                | Self::ShapeComponent
                | Self::ShapeComponentPicture
                | Self::EqEdit
        )
    }

    /// 태그 이름 반환 (디버깅용)
    pub fn name(&self) -> &'static str {
        match self {
            Self::DocumentProperties => "DOCUMENT_PROPERTIES",
            Self::IdMappings => "ID_MAPPINGS",
            Self::BinData => "BIN_DATA",
            Self::FaceName => "FACE_NAME",
            Self::BorderFill => "BORDER_FILL",
            Self::CharShape => "CHAR_SHAPE",
            Self::TabDef => "TAB_DEF",
            Self::ParaShape => "PARA_SHAPE",
            Self::Style => "STYLE",
            Self::MemoShape => "MEMO_SHAPE",
            Self::DocData => "DOC_DATA",
            Self::DistributeDocData => "DISTRIBUTE_DOC_DATA",
            Self::CompatibleDocument => "COMPATIBLE_DOCUMENT",
            Self::LayoutCompatibility => "LAYOUT_COMPATIBILITY",
            Self::TrackChange => "TRACK_CHANGE",
            Self::TrackChangeAuthor => "TRACK_CHANGE_AUTHOR",
            Self::ParaHeader => "PARA_HEADER",
            Self::ParaText => "PARA_TEXT",
            Self::ParaCharShape => "PARA_CHAR_SHAPE",
            Self::ParaLineSeg => "PARA_LINE_SEG",
            Self::ParaRangeTag => "PARA_RANGE_TAG",
            Self::CtrlHeader => "CTRL_HEADER",
            Self::ListHeader => "LIST_HEADER",
            Self::PageDef => "PAGE_DEF",
            Self::FootnoteShape => "FOOTNOTE_SHAPE",
            Self::PageBorderFill => "PAGE_BORDER_FILL",
            Self::Table => "TABLE",
            Self::TableCell => "TABLE_CELL",
            Self::ShapeComponent => "SHAPE_COMPONENT",
            Self::ShapeComponentLine => "SHAPE_COMPONENT_LINE",
            Self::ShapeComponentRectangle => "SHAPE_COMPONENT_RECTANGLE",
            Self::ShapeComponentEllipse => "SHAPE_COMPONENT_ELLIPSE",
            Self::ShapeComponentArc => "SHAPE_COMPONENT_ARC",
            Self::ShapeComponentPolygon => "SHAPE_COMPONENT_POLYGON",
            Self::ShapeComponentCurve => "SHAPE_COMPONENT_CURVE",
            Self::ShapeComponentPicture => "SHAPE_COMPONENT_PICTURE",
            Self::ShapeComponentOle => "SHAPE_COMPONENT_OLE",
            Self::ShapeComponentContainer => "SHAPE_COMPONENT_CONTAINER",
            Self::CtrlData => "CTRL_DATA",
            Self::FormObject => "FORM_OBJECT",
            Self::EqEdit => "EQ_EDIT",
            Self::Unknown(_) => "UNKNOWN",
        }
    }
}

impl From<u16> for RecordTag {
    fn from(value: u16) -> Self {
        match value {
            // DocInfo
            0x00 => Self::DocumentProperties,
            0x01 => Self::IdMappings,
            0x02 => Self::BinData,
            0x03 => Self::FaceName,
            0x04 => Self::BorderFill,
            0x07 => Self::CharShape,
            0x08 => Self::TabDef,
            0x09 => Self::ParaShape,
            0x0A => Self::Style,
            0x0B => Self::MemoShape,
            0x0C => Self::DocData,
            0x0D => Self::DistributeDocData,
            0x12 => Self::CompatibleDocument,
            0x13 => Self::LayoutCompatibility,
            0x14 => Self::TrackChange,
            0x15 => Self::TrackChangeAuthor,

            // BodyText
            0x42 => Self::ParaHeader,
            0x43 => Self::ParaText,
            0x44 => Self::ParaCharShape,
            0x45 => Self::ParaLineSeg,
            0x46 => Self::ParaRangeTag,
            0x47 => Self::CtrlHeader,
            0x4F => Self::ListHeader,
            0x50 => Self::PageDef,
            0x51 => Self::FootnoteShape,
            0x52 => Self::PageBorderFill,

            // Table
            0x4D => Self::Table,
            0x4E => Self::TableCell,

            // Shape
            0x4B => Self::ShapeComponent,
            0x53 => Self::ShapeComponentRectangle,
            0x54 => Self::ShapeComponentEllipse,
            0x55 => Self::ShapeComponentArc,
            0x56 => Self::ShapeComponentPolygon,
            0x57 => Self::ShapeComponentCurve,
            0x58 => Self::ShapeComponentPicture,
            0x59 => Self::ShapeComponentOle,
            0x5A => Self::ShapeComponentContainer,

            // Control
            0x48 => Self::CtrlData,
            0x60 => Self::FormObject,
            0x61 => Self::EqEdit,

            other => Self::Unknown(other),
        }
    }
}

impl std::fmt::Display for RecordTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(0x{:02X})", self.name(), self.to_u16())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 레거시 상수 (하위 호환성)
// ═══════════════════════════════════════════════════════════════════════════

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

    // ═══════════════════════════════════════════════════════════════
    // 레거시 상수 테스트
    // ═══════════════════════════════════════════════════════════════

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
    #[allow(clippy::assertions_on_constants)]
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

    // ═══════════════════════════════════════════════════════════════
    // RecordTag Enum 테스트
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_convert_u16_to_record_tag() {
        // Arrange & Act & Assert
        assert_eq!(RecordTag::from(0x00), RecordTag::DocumentProperties);
        assert_eq!(RecordTag::from(0x03), RecordTag::FaceName);
        assert_eq!(RecordTag::from(0x07), RecordTag::CharShape);
        assert_eq!(RecordTag::from(0x09), RecordTag::ParaShape);
        assert_eq!(RecordTag::from(0x42), RecordTag::ParaHeader);
        assert_eq!(RecordTag::from(0x43), RecordTag::ParaText);
        assert_eq!(RecordTag::from(0x4D), RecordTag::Table);
        assert_eq!(RecordTag::from(0xFF), RecordTag::Unknown(0xFF));
    }

    #[test]
    fn test_should_convert_record_tag_to_u16() {
        // Arrange & Act & Assert
        assert_eq!(RecordTag::DocumentProperties.to_u16(), 0x00);
        assert_eq!(RecordTag::FaceName.to_u16(), 0x03);
        assert_eq!(RecordTag::CharShape.to_u16(), 0x07);
        assert_eq!(RecordTag::ParaShape.to_u16(), 0x09);
        assert_eq!(RecordTag::ParaHeader.to_u16(), 0x42);
        assert_eq!(RecordTag::ParaText.to_u16(), 0x43);
        assert_eq!(RecordTag::Table.to_u16(), 0x4D);
        assert_eq!(RecordTag::Unknown(0xFF).to_u16(), 0xFF);
    }

    #[test]
    fn test_should_identify_docinfo_tags_via_enum() {
        // Arrange & Act & Assert
        assert!(RecordTag::DocumentProperties.is_docinfo());
        assert!(RecordTag::CharShape.is_docinfo());
        assert!(RecordTag::ParaShape.is_docinfo());
        assert!(RecordTag::FaceName.is_docinfo());
        assert!(!RecordTag::ParaHeader.is_docinfo());
        assert!(!RecordTag::ParaText.is_docinfo());
        assert!(!RecordTag::Table.is_docinfo());
    }

    #[test]
    fn test_should_identify_bodytext_tags_via_enum() {
        // Arrange & Act & Assert
        assert!(RecordTag::ParaHeader.is_bodytext());
        assert!(RecordTag::ParaText.is_bodytext());
        assert!(RecordTag::Table.is_bodytext());
        assert!(RecordTag::CtrlHeader.is_bodytext());
        assert!(!RecordTag::DocumentProperties.is_bodytext());
        assert!(!RecordTag::CharShape.is_bodytext());
    }

    #[test]
    fn test_should_identify_control_tags() {
        // Arrange & Act & Assert
        assert!(RecordTag::Table.is_control());
        assert!(RecordTag::ShapeComponentPicture.is_control());
        assert!(!RecordTag::ParaText.is_control());
        assert!(!RecordTag::CharShape.is_control());
    }

    #[test]
    fn test_should_return_tag_name() {
        // Arrange & Act & Assert
        assert_eq!(RecordTag::ParaText.name(), "PARA_TEXT");
        assert_eq!(RecordTag::CharShape.name(), "CHAR_SHAPE");
        assert_eq!(RecordTag::Table.name(), "TABLE");
        assert_eq!(RecordTag::Unknown(0xFF).name(), "UNKNOWN");
    }

    #[test]
    fn test_should_display_tag_with_hex() {
        // Arrange & Act
        let display = format!("{}", RecordTag::ParaText);

        // Assert
        assert_eq!(display, "PARA_TEXT(0x43)");
    }

    #[test]
    fn test_should_serialize_record_tag_to_json() {
        // Arrange
        let tag = RecordTag::ParaText;

        // Act
        let json = serde_json::to_string(&tag).unwrap();
        let restored: RecordTag = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(tag, restored);
    }

    #[test]
    fn test_should_roundtrip_unknown_tag() {
        // Arrange
        let tag = RecordTag::Unknown(0x99);

        // Act
        let json = serde_json::to_string(&tag).unwrap();
        let restored: RecordTag = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(tag, restored);
        assert_eq!(restored.to_u16(), 0x99);
    }
}
