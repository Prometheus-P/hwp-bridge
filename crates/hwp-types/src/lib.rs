// crates/hwp-types/src/lib.rs

//! HWP 타입 정의 크레이트
//!
//! HWP 문서 파싱 및 변환에 사용되는 공용 타입 정의입니다.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// === Modules ===
pub mod bindata;
pub mod border_fill;
pub mod control;
pub mod document;
pub mod face_name;
pub mod record;
pub mod structured;
pub mod style;
pub mod tags;

// === Re-exports ===
pub use bindata::{BinData, BinDataType};
pub use border_fill::{
    BorderFill, BorderLine, BorderLineType, FillGradient, FillImage, FillInfo, GradientType,
};
pub use control::{Control, Picture, Table, TableCell};
pub use document::{Paragraph, Section};
pub use face_name::{FaceName, FontLanguage, Panose, SubstituteFontType};
pub use record::RecordHeader;
pub use structured::{
    ContentBlock, ContentLocation, InlineStyle, OutlineItem, PageOrientation, PageSetup,
    ParagraphType, StructuredDocument, StructuredEquation, StructuredFootnote, StructuredImage,
    StructuredMetadata, StructuredParagraph, StructuredSection, StructuredTable,
    StructuredTableCell, StyleDefinitions, TextAlignment, TextRun,
};
pub use style::{Alignment, CharShape, CharShapeAttr, LineSpaceType, ParaShape, ParaShapeAttr};
pub use tags::RecordTag;

/// HWP Document File 시그니처 (32 bytes, null-padded)
pub const HWP_SIGNATURE: &[u8; 32] = b"HWP Document File\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

/// HWP 변환 프로젝트 전반에서 사용하는 공용 에러 타입
#[derive(Error, Debug)]
pub enum HwpError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OLE Storage Error: {0}")]
    OleError(String),

    #[error("Invalid HWP Signature")]
    InvalidSignature,

    #[error("Unsupported HWP Version: {0}")]
    UnsupportedVersion(String),

    #[error("Encrypted Document (Cannot Process)")]
    Encrypted,

    #[error("Distribution Document (Read-Only/Encrypted Body)")]
    DistributionOnly,

    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Google Drive API Error: {0}")]
    GoogleDriveError(String),
}

/// HWP 파일 버전 (예: 5.1.0.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HwpVersion {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub revision: u8,
}

impl HwpVersion {
    pub fn new(major: u8, minor: u8, build: u8, revision: u8) -> Self {
        Self {
            major,
            minor,
            build,
            revision,
        }
    }

    /// 버전 바이트(4 bytes, little-endian)에서 파싱
    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        Self {
            // HWP stores version as: [revision, build, minor, major] in LE
            major: bytes[3],
            minor: bytes[2],
            build: bytes[1],
            revision: bytes[0],
        }
    }

    /// HWP 5.0 이상인지 확인
    pub fn is_supported(&self) -> bool {
        self.major >= 5
    }
}

impl std::fmt::Display for HwpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.major, self.minor, self.build, self.revision
        )
    }
}

/// 문서 속성 비트 플래그
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentProperties(u32);

impl DocumentProperties {
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    /// Bit 0: 압축 여부
    pub fn is_compressed(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    /// Bit 1: 암호화 여부
    pub fn is_encrypted(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// Bit 2: 배포용 문서
    pub fn is_distribution(&self) -> bool {
        self.0 & (1 << 2) != 0
    }

    /// Bit 3: 스크립트 저장
    pub fn has_script(&self) -> bool {
        self.0 & (1 << 3) != 0
    }

    /// Bit 4: DRM 보안
    pub fn has_drm(&self) -> bool {
        self.0 & (1 << 4) != 0
    }

    /// Bit 5: XMLTemplate 스토리지 존재
    pub fn has_xml_template(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Bit 6: 문서 이력 관리
    pub fn has_history(&self) -> bool {
        self.0 & (1 << 6) != 0
    }

    /// Bit 7: 전자 서명 정보 존재
    pub fn has_signature(&self) -> bool {
        self.0 & (1 << 7) != 0
    }

    /// Bit 8: 공인 인증서 암호화
    pub fn has_cert_encryption(&self) -> bool {
        self.0 & (1 << 8) != 0
    }

    /// Bit 11: CCL 문서
    pub fn is_ccl(&self) -> bool {
        self.0 & (1 << 11) != 0
    }

    /// Bit 12: 모바일 최적화
    pub fn is_mobile_optimized(&self) -> bool {
        self.0 & (1 << 12) != 0
    }

    /// Bit 14: 변경 추적
    pub fn has_track_changes(&self) -> bool {
        self.0 & (1 << 14) != 0
    }

    /// Bit 15: 공공누리(KOGL) 저작권
    pub fn is_kogl(&self) -> bool {
        self.0 & (1 << 15) != 0
    }
}

/// HWP FileHeader (256 bytes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeader {
    /// 파일 버전
    pub version: HwpVersion,
    /// 문서 속성
    pub properties: DocumentProperties,
}

impl FileHeader {
    /// FileHeader가 처리 가능한 문서인지 검증 (Fail-Fast)
    pub fn validate(&self) -> Result<(), HwpError> {
        // 버전 검증
        if !self.version.is_supported() {
            return Err(HwpError::UnsupportedVersion(self.version.to_string()));
        }

        // 암호화 문서 검증
        if self.properties.is_encrypted() {
            return Err(HwpError::Encrypted);
        }

        // 배포용 문서 검증
        if self.properties.is_distribution() {
            return Err(HwpError::DistributionOnly);
        }

        Ok(())
    }
}

/// 파싱 결과물 최상위 구조체
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HwpDocument {
    /// 문서 메타데이터
    pub metadata: DocumentMetadata,
    /// 문서 섹션 목록
    pub sections: Vec<Section>,
    /// 글자 모양 목록 (DocInfo에서 파싱)
    pub char_shapes: Vec<CharShape>,
    /// 문단 모양 목록 (DocInfo에서 파싱)
    pub para_shapes: Vec<ParaShape>,
    /// 바이너리 데이터 목록 (이미지, OLE 객체 등)
    pub bin_data: Vec<BinData>,
}

impl HwpDocument {
    /// 새 빈 문서 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 섹션 추가
    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    /// 바이너리 데이터 추가
    pub fn add_bin_data(&mut self, data: BinData) {
        self.bin_data.push(data);
    }

    /// 전체 텍스트 추출 (간소화 버전)
    pub fn extract_text(&self) -> String {
        self.sections
            .iter()
            .flat_map(|s| s.paragraphs.iter())
            .map(|p| p.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub created_at: String,
    pub is_encrypted: bool,
    pub is_distribution: bool,
}

/// 변환 옵션
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub extract_images: bool,
    pub include_comments: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// T048: JSON serialization roundtrip test for all major types
    #[test]
    fn test_should_roundtrip_hwp_document_when_serialized_to_json() {
        // Arrange - Create a complete HwpDocument
        let mut doc = HwpDocument::new();
        doc.metadata.title = "Test Document".to_string();
        doc.metadata.author = "Test Author".to_string();

        // Add a section with paragraph
        let mut section = Section::new();
        let mut para = Paragraph::new("Hello, HWP!");
        para.add_char_shape(0, 1);
        section.push_paragraph(para);
        doc.add_section(section);

        // Add char_shapes
        let mut char_shape = CharShape::default();
        char_shape.base_size = 1000;
        char_shape.text_color = 0x000000;
        char_shape.attr = CharShapeAttr::from_bits(0b11); // bold + italic
        doc.char_shapes.push(char_shape);

        // Add para_shapes
        let mut para_shape = ParaShape::default();
        para_shape.margin_left = 100;
        para_shape.attr = ParaShapeAttr::from_bits(0b1100); // center alignment
        doc.para_shapes.push(para_shape);

        // Add bin_data
        let bin_data = BinData::new(1, BinDataType::Embedding)
            .with_extension("png")
            .with_data(vec![0x89, 0x50, 0x4E, 0x47]); // PNG magic bytes
        doc.add_bin_data(bin_data);

        // Act - Serialize and deserialize
        let json = serde_json::to_string(&doc).expect("Failed to serialize HwpDocument");
        let deserialized: HwpDocument =
            serde_json::from_str(&json).expect("Failed to deserialize HwpDocument");

        // Assert
        assert_eq!(doc.metadata.title, deserialized.metadata.title);
        assert_eq!(doc.metadata.author, deserialized.metadata.author);
        assert_eq!(doc.sections.len(), deserialized.sections.len());
        assert_eq!(
            doc.sections[0].paragraphs[0].text,
            deserialized.sections[0].paragraphs[0].text
        );
        assert_eq!(doc.char_shapes.len(), deserialized.char_shapes.len());
        assert!(deserialized.char_shapes[0].attr.is_bold());
        assert!(deserialized.char_shapes[0].attr.is_italic());
        assert_eq!(doc.para_shapes.len(), deserialized.para_shapes.len());
        assert_eq!(
            deserialized.para_shapes[0].attr.alignment(),
            Alignment::Center
        );
        assert_eq!(doc.bin_data.len(), deserialized.bin_data.len());
        assert_eq!(
            doc.bin_data[0].extension,
            deserialized.bin_data[0].extension
        );
    }

    #[test]
    fn test_should_roundtrip_record_header_when_serialized() {
        // Arrange
        let header = RecordHeader::new(0x42, 5, 1000);

        // Act
        let json = serde_json::to_string(&header).expect("Failed to serialize");
        let deserialized: RecordHeader =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(header, deserialized);
    }

    #[test]
    fn test_should_roundtrip_control_table_when_serialized() {
        // Arrange
        let mut table = Table::new(2, 3);
        table.add_cell(TableCell::new(0, 0).with_span(1, 1));
        let control = Control::Table(table);

        // Act
        let json = serde_json::to_string(&control).expect("Failed to serialize");
        let deserialized: Control = serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        if let Control::Table(t) = deserialized {
            assert_eq!(t.rows, 2);
            assert_eq!(t.cols, 3);
            assert_eq!(t.cells.len(), 1);
        } else {
            panic!("Expected Control::Table");
        }
    }

    #[test]
    fn test_should_roundtrip_file_header_when_serialized() {
        // Arrange
        let header = FileHeader {
            version: HwpVersion::new(5, 1, 0, 0),
            properties: DocumentProperties::from_bits(0b001), // compressed
        };

        // Act
        let json = serde_json::to_string(&header).expect("Failed to serialize");
        let deserialized: FileHeader = serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(header.version.major, deserialized.version.major);
        assert!(deserialized.properties.is_compressed());
    }
}
