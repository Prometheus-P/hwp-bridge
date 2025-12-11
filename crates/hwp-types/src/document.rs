// crates/hwp-types/src/document.rs
//! 문서 구조 타입
//!
//! HWP 문서의 계층 구조(Section, Paragraph)를 표현합니다.

use serde::{Deserialize, Serialize};

/// 문서 섹션
///
/// 하나의 HWP 문서는 여러 섹션으로 구성됩니다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section {
    /// 섹션 내 문단 목록
    pub paragraphs: Vec<Paragraph>,
}

impl Section {
    /// 새 빈 섹션 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 문단 추가
    pub fn push_paragraph(&mut self, paragraph: Paragraph) {
        self.paragraphs.push(paragraph);
    }
}

/// 문단
///
/// 텍스트와 스타일 참조, 인라인 컨트롤을 포함합니다.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Paragraph {
    /// 문단 텍스트
    pub text: String,
    /// 문단 모양 ID 참조 (DocInfo의 ParaShape 인덱스)
    pub para_shape_id: u16,
    /// 글자 위치별 CharShape ID: (position, shape_id)
    pub char_shapes: Vec<(u32, u16)>,
}

impl Paragraph {
    /// 새 문단 생성
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            para_shape_id: 0,
            char_shapes: Vec::new(),
        }
    }

    /// 문단 모양 ID 설정
    pub fn with_para_shape_id(mut self, id: u16) -> Self {
        self.para_shape_id = id;
        self
    }

    /// CharShape 참조 추가
    pub fn add_char_shape(&mut self, position: u32, shape_id: u16) {
        self.char_shapes.push((position, shape_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_should_create_empty_section_when_default() {
        // Arrange & Act
        let section = Section::default();

        // Assert
        assert!(section.paragraphs.is_empty());
    }

    #[test]
    fn test_should_create_empty_section_when_new() {
        // Arrange & Act
        let section = Section::new();

        // Assert
        assert!(section.paragraphs.is_empty());
    }

    #[test]
    fn test_should_store_paragraph_text_when_created() {
        // Arrange & Act
        let paragraph = Paragraph::new("Hello, HWP!");

        // Assert
        assert_eq!(paragraph.text, "Hello, HWP!");
        assert_eq!(paragraph.para_shape_id, 0);
        assert!(paragraph.char_shapes.is_empty());
    }

    #[test]
    fn test_should_add_paragraph_to_section_when_pushed() {
        // Arrange
        let mut section = Section::new();
        let paragraph = Paragraph::new("First paragraph");

        // Act
        section.push_paragraph(paragraph);

        // Assert
        assert_eq!(section.paragraphs.len(), 1);
        assert_eq!(section.paragraphs[0].text, "First paragraph");
    }

    #[test]
    fn test_should_set_para_shape_id_when_builder_used() {
        // Arrange & Act
        let paragraph = Paragraph::new("Styled text").with_para_shape_id(5);

        // Assert
        assert_eq!(paragraph.para_shape_id, 5);
    }

    #[test]
    fn test_should_add_char_shape_reference_when_added() {
        // Arrange
        let mut paragraph = Paragraph::new("Mixed styles");

        // Act
        paragraph.add_char_shape(0, 1);
        paragraph.add_char_shape(5, 2);

        // Assert
        assert_eq!(paragraph.char_shapes.len(), 2);
        assert_eq!(paragraph.char_shapes[0], (0, 1));
        assert_eq!(paragraph.char_shapes[1], (5, 2));
    }

    #[test]
    fn test_should_serialize_section_to_json_when_serde_used() {
        // Arrange
        let mut section = Section::new();
        section.push_paragraph(Paragraph::new("Test"));

        // Act
        let json = serde_json::to_string(&section).unwrap();
        let deserialized: Section = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(section.paragraphs.len(), deserialized.paragraphs.len());
        assert_eq!(section.paragraphs[0].text, deserialized.paragraphs[0].text);
    }

    #[test]
    fn test_should_handle_empty_document_when_no_sections() {
        // Arrange & Act
        let sections: Vec<Section> = Vec::new();

        // Assert
        assert!(sections.is_empty());
    }
}
