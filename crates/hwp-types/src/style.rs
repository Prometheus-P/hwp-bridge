// crates/hwp-types/src/style.rs
//! 스타일 타입
//!
//! 글자 모양(CharShape)과 문단 모양(ParaShape)을 정의합니다.

use serde::{Deserialize, Serialize};

/// 문단 정렬
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    /// 왼쪽 정렬
    #[default]
    Left = 0,
    /// 가운데 정렬
    Center = 1,
    /// 오른쪽 정렬
    Right = 2,
    /// 양쪽 정렬
    Justify = 3,
    /// 배분 정렬
    Distribute = 4,
}

/// 글자 속성 비트 플래그
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharShapeAttr(pub u32);

impl CharShapeAttr {
    /// 새 CharShapeAttr 생성
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// 원시 비트 값 반환
    pub fn bits(&self) -> u32 {
        self.0
    }

    /// Bit 0: 굵게
    pub fn is_bold(&self) -> bool {
        self.0 & (1 << 0) != 0
    }

    /// Bit 1: 기울임
    pub fn is_italic(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// Bit 2-3: 밑줄 종류 (0=없음, 1=실선, 2=점선, ...)
    pub fn underline_type(&self) -> u8 {
        ((self.0 >> 2) & 0x03) as u8
    }

    /// Bit 4-5: 외곽선 종류
    pub fn outline_type(&self) -> u8 {
        ((self.0 >> 4) & 0x03) as u8
    }

    /// Bit 6-7: 그림자 종류
    pub fn shadow_type(&self) -> u8 {
        ((self.0 >> 6) & 0x03) as u8
    }

    /// Bit 8: 양각
    pub fn is_emboss(&self) -> bool {
        self.0 & (1 << 8) != 0
    }

    /// Bit 9: 음각
    pub fn is_engrave(&self) -> bool {
        self.0 & (1 << 9) != 0
    }

    /// Bit 10: 위 첨자
    pub fn is_superscript(&self) -> bool {
        self.0 & (1 << 10) != 0
    }

    /// Bit 11: 아래 첨자
    pub fn is_subscript(&self) -> bool {
        self.0 & (1 << 11) != 0
    }

    /// Bit 12-14: 취소선 종류
    pub fn strikethrough_type(&self) -> u8 {
        ((self.0 >> 12) & 0x07) as u8
    }
}

/// 글자 모양 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharShape {
    /// 글꼴 ID 참조 (언어별 7개: 한글, 영문, 한자, 일문, 기타, 기호, 사용자)
    pub font_ids: [u16; 7],
    /// 글꼴 비율 (%, 언어별)
    pub font_scales: [u8; 7],
    /// 자간 (언어별)
    pub char_spacing: [i8; 7],
    /// 상대 크기 (%, 언어별)
    pub relative_sizes: [u8; 7],
    /// 기준 크기 (1/100 pt 단위)
    pub base_size: u32,
    /// 글자 색상 (COLORREF: 0x00BBGGRR)
    pub text_color: u32,
    /// 밑줄 색상
    pub underline_color: u32,
    /// 음영 색상
    pub shade_color: u32,
    /// 그림자 색상
    pub shadow_color: u32,
    /// 속성 플래그 (Bold, Italic, Underline 등)
    pub attr: CharShapeAttr,
}

/// 문단 속성 비트 플래그
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParaShapeAttr(pub u32);

impl ParaShapeAttr {
    /// 새 ParaShapeAttr 생성
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// 원시 비트 값 반환
    pub fn bits(&self) -> u32 {
        self.0
    }

    /// Bit 0-1: 줄 간격 종류 (0=%, 1=고정, 2=여백만, 3=최소)
    pub fn line_spacing_type(&self) -> u8 {
        (self.0 & 0x03) as u8
    }

    /// Bit 2-4: 정렬 (0=양쪽, 1=왼쪽, 2=오른쪽, 3=가운데, 4=배분, 5=나눔)
    pub fn alignment(&self) -> Alignment {
        match (self.0 >> 2) & 0x07 {
            0 => Alignment::Justify,
            1 => Alignment::Left,
            2 => Alignment::Right,
            3 => Alignment::Center,
            4 => Alignment::Distribute,
            _ => Alignment::Left,
        }
    }
}

/// 문단 모양 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParaShape {
    /// 속성 플래그
    pub attr: ParaShapeAttr,
    /// 왼쪽 여백 (HWPUNIT: 1/7200 inch)
    pub margin_left: i32,
    /// 오른쪽 여백
    pub margin_right: i32,
    /// 들여쓰기 (양수: 들여쓰기, 음수: 내어쓰기)
    pub indent: i32,
    /// 문단 위 간격
    pub margin_top: i32,
    /// 문단 아래 간격
    pub margin_bottom: i32,
    /// 줄 간격 (% 또는 고정값)
    pub line_spacing: i32,
    /// 탭 정의 ID
    pub tab_def_id: u16,
    /// 번호/글머리 ID
    pub para_head_id: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    // === CharShapeAttr Tests ===

    #[test]
    fn test_should_return_true_when_bold_bit_set() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0001); // Bit 0 set

        // Act & Assert
        assert!(attr.is_bold());
    }

    #[test]
    fn test_should_return_false_when_bold_bit_not_set() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0010); // Bit 1 set, not bit 0

        // Act & Assert
        assert!(!attr.is_bold());
    }

    #[test]
    fn test_should_return_true_when_italic_bit_set() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0010); // Bit 1 set

        // Act & Assert
        assert!(attr.is_italic());
    }

    #[test]
    fn test_should_return_false_when_italic_bit_not_set() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0001); // Bit 0 set, not bit 1

        // Act & Assert
        assert!(!attr.is_italic());
    }

    #[test]
    fn test_should_detect_bold_and_italic_when_both_set() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0011); // Bit 0 and 1 set

        // Act & Assert
        assert!(attr.is_bold());
        assert!(attr.is_italic());
    }

    #[test]
    fn test_should_return_underline_type_when_bits_extracted() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0000_0100); // Bit 2 set (underline type = 1)

        // Act & Assert
        assert_eq!(attr.underline_type(), 1);
    }

    #[test]
    fn test_should_return_strikethrough_type_when_bits_extracted() {
        // Arrange
        let attr = CharShapeAttr::from_bits(0b0011_0000_0000_0000); // Bits 12-14 = 3

        // Act & Assert
        assert_eq!(attr.strikethrough_type(), 3);
    }

    #[test]
    fn test_should_create_default_charshape_when_default() {
        // Arrange & Act
        let shape = CharShape::default();

        // Assert
        assert_eq!(shape.font_ids, [0; 7]);
        assert_eq!(shape.base_size, 0);
        assert_eq!(shape.text_color, 0);
        assert!(!shape.attr.is_bold());
    }

    // === ParaShapeAttr Tests ===

    #[test]
    fn test_should_return_alignment_justify_when_bits_zero() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0000_0000); // Bits 2-4 = 0

        // Act & Assert
        assert_eq!(attr.alignment(), Alignment::Justify);
    }

    #[test]
    fn test_should_return_alignment_left_when_bits_match() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0000_0100); // Bits 2-4 = 1

        // Act & Assert
        assert_eq!(attr.alignment(), Alignment::Left);
    }

    #[test]
    fn test_should_return_alignment_center_when_bits_match() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0000_1100); // Bits 2-4 = 3

        // Act & Assert
        assert_eq!(attr.alignment(), Alignment::Center);
    }

    #[test]
    fn test_should_return_alignment_right_when_bits_match() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0000_1000); // Bits 2-4 = 2

        // Act & Assert
        assert_eq!(attr.alignment(), Alignment::Right);
    }

    #[test]
    fn test_should_return_alignment_distribute_when_bits_match() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0001_0000); // Bits 2-4 = 4

        // Act & Assert
        assert_eq!(attr.alignment(), Alignment::Distribute);
    }

    #[test]
    fn test_should_return_line_spacing_type_when_extracted() {
        // Arrange
        let attr = ParaShapeAttr::from_bits(0b0000_0010); // Bits 0-1 = 2

        // Act & Assert
        assert_eq!(attr.line_spacing_type(), 2);
    }

    #[test]
    fn test_should_create_default_parashape_when_default() {
        // Arrange & Act
        let shape = ParaShape::default();

        // Assert
        assert_eq!(shape.margin_left, 0);
        assert_eq!(shape.margin_right, 0);
        assert_eq!(shape.line_spacing, 0);
        assert_eq!(shape.attr.alignment(), Alignment::Justify);
    }

    // === Serialization Tests ===

    #[test]
    fn test_should_serialize_charshape_to_json_when_serde_used() {
        // Arrange
        let mut shape = CharShape::default();
        shape.base_size = 1000;
        shape.attr = CharShapeAttr::from_bits(0b0000_0011);

        // Act
        let json = serde_json::to_string(&shape).unwrap();
        let deserialized: CharShape = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(shape.base_size, deserialized.base_size);
        assert!(deserialized.attr.is_bold());
        assert!(deserialized.attr.is_italic());
    }

    #[test]
    fn test_should_serialize_parashape_to_json_when_serde_used() {
        // Arrange
        let mut shape = ParaShape::default();
        shape.margin_left = 1000;
        shape.attr = ParaShapeAttr::from_bits(0b0000_1100); // Center

        // Act
        let json = serde_json::to_string(&shape).unwrap();
        let deserialized: ParaShape = serde_json::from_str(&json).unwrap();

        // Assert
        assert_eq!(shape.margin_left, deserialized.margin_left);
        assert_eq!(deserialized.attr.alignment(), Alignment::Center);
    }
}
