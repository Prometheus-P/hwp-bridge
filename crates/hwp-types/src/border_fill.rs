// crates/hwp-types/src/border_fill.rs

//! 테두리/배경 타입
//!
//! HWP 문서의 테두리(Border)와 채우기(Fill) 정보를 정의합니다.

use serde::{Deserialize, Serialize};

/// 테두리/배경 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BorderFill {
    /// 속성 플래그
    pub properties: u16,
    /// 왼쪽 테두리
    pub left: BorderLine,
    /// 오른쪽 테두리
    pub right: BorderLine,
    /// 위쪽 테두리
    pub top: BorderLine,
    /// 아래쪽 테두리
    pub bottom: BorderLine,
    /// 대각선 테두리
    pub diagonal: BorderLine,
    /// 채우기 정보
    pub fill: FillInfo,
}

impl BorderFill {
    /// 새 BorderFill 생성 (기본값)
    pub fn new() -> Self {
        Self::default()
    }

    /// 테두리가 있는 BorderFill 생성
    pub fn with_border() -> Self {
        let border = BorderLine::solid(0x000000); // 검정 실선
        Self {
            properties: 0x0001,
            left: border.clone(),
            right: border.clone(),
            top: border.clone(),
            bottom: border,
            diagonal: BorderLine::none(),
            fill: FillInfo::solid(0xFFFFFFFF), // 흰색 배경
        }
    }

    /// 3D 효과 여부 (Bit 0)
    pub fn has_3d_effect(&self) -> bool {
        self.properties & 0x01 != 0
    }

    /// 그림자 효과 여부 (Bit 1)
    pub fn has_shadow(&self) -> bool {
        self.properties & 0x02 != 0
    }

    /// 대각선 방향 (Bits 2-3)
    /// 0: 없음, 1: ↘, 2: ↗, 3: 교차
    pub fn diagonal_direction(&self) -> u8 {
        ((self.properties >> 2) & 0x03) as u8
    }
}

/// 테두리 선
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BorderLine {
    /// 선 종류 (0=없음, 1=실선, 2=파선, 3=점선, ...)
    pub line_type: BorderLineType,
    /// 선 두께 (0.1mm 단위)
    pub thickness: u8,
    /// 선 색상 (COLORREF: 0x00BBGGRR)
    pub color: u32,
}

impl BorderLine {
    /// 테두리 없음
    pub fn none() -> Self {
        Self::default()
    }

    /// 실선 테두리
    pub fn solid(color: u32) -> Self {
        Self {
            line_type: BorderLineType::Solid,
            thickness: 1, // 0.1mm
            color,
        }
    }

    /// 두꺼운 실선 테두리
    pub fn thick(color: u32, thickness: u8) -> Self {
        Self {
            line_type: BorderLineType::Solid,
            thickness,
            color,
        }
    }

    /// 테두리가 있는지 확인
    pub fn is_visible(&self) -> bool {
        self.line_type != BorderLineType::None && self.thickness > 0
    }
}

/// 테두리 선 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BorderLineType {
    /// 없음
    #[default]
    None = 0,
    /// 실선
    Solid = 1,
    /// 파선
    Dash = 2,
    /// 점선
    Dot = 3,
    /// 파선-점선
    DashDot = 4,
    /// 파선-점선-점선
    DashDotDot = 5,
    /// 긴 파선
    LongDash = 6,
    /// 원형 점선
    CircleDot = 7,
    /// 이중선
    Double = 8,
    /// 가는선-굵은선 이중선
    ThinThick = 9,
    /// 굵은선-가는선 이중선
    ThickThin = 10,
    /// 가는선-굵은선-가는선 삼중선
    ThinThickThin = 11,
    /// 물결선
    Wave = 12,
    /// 이중 물결선
    DoubleWave = 13,
    /// 굵은 삼중선
    ThickTriple = 14,
    /// 가는 삼중선
    ThinTriple = 15,
}

impl BorderLineType {
    /// u8에서 BorderLineType으로 변환
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Solid,
            2 => Self::Dash,
            3 => Self::Dot,
            4 => Self::DashDot,
            5 => Self::DashDotDot,
            6 => Self::LongDash,
            7 => Self::CircleDot,
            8 => Self::Double,
            9 => Self::ThinThick,
            10 => Self::ThickThin,
            11 => Self::ThinThickThin,
            12 => Self::Wave,
            13 => Self::DoubleWave,
            14 => Self::ThickTriple,
            15 => Self::ThinTriple,
            _ => Self::None,
        }
    }
}

/// 채우기 정보
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FillInfo {
    /// 채우기 종류 플래그
    /// Bit 0: 단색 채우기
    /// Bit 1: 그라데이션
    /// Bit 2: 이미지
    pub fill_type: u32,
    /// 배경색 (COLORREF)
    pub background_color: u32,
    /// 패턴 색상
    pub pattern_color: u32,
    /// 패턴 종류
    pub pattern_type: u32,
    /// 이미지 정보 (이미지 채우기인 경우)
    pub image: Option<FillImage>,
    /// 그라데이션 정보 (그라데이션 채우기인 경우)
    pub gradient: Option<FillGradient>,
}

impl FillInfo {
    /// 채우기 없음
    pub fn none() -> Self {
        Self::default()
    }

    /// 단색 채우기
    pub fn solid(color: u32) -> Self {
        Self {
            fill_type: 0x01,
            background_color: color,
            ..Default::default()
        }
    }

    /// 단색 채우기 여부
    pub fn is_solid(&self) -> bool {
        self.fill_type & 0x01 != 0
    }

    /// 그라데이션 채우기 여부
    pub fn is_gradient(&self) -> bool {
        self.fill_type & 0x02 != 0
    }

    /// 이미지 채우기 여부
    pub fn is_image(&self) -> bool {
        self.fill_type & 0x04 != 0
    }
}

/// 이미지 채우기 정보
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FillImage {
    /// 밝기 (-100 ~ 100)
    pub brightness: i8,
    /// 대비 (-100 ~ 100)
    pub contrast: i8,
    /// 효과 (0=원본, 1=그레이스케일, 2=흑백)
    pub effect: u8,
    /// BinData ID 참조
    pub bin_data_id: u16,
}

/// 그라데이션 채우기 정보
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FillGradient {
    /// 그라데이션 종류 (0=선형, 1=방사형, 2=원뿔형, 3=사각형)
    pub gradient_type: GradientType,
    /// 시작 색상
    pub start_color: u32,
    /// 끝 색상
    pub end_color: u32,
    /// 각도 (0-360도)
    pub angle: u16,
    /// 중심 X (0-100%)
    pub center_x: u16,
    /// 중심 Y (0-100%)
    pub center_y: u16,
    /// 번짐 정도
    pub blur: u16,
}

/// 그라데이션 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum GradientType {
    /// 선형
    #[default]
    Linear = 0,
    /// 방사형
    Radial = 1,
    /// 원뿔형
    Conical = 2,
    /// 사각형
    Square = 3,
}

impl GradientType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::Radial,
            2 => Self::Conical,
            3 => Self::Square,
            _ => Self::Linear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // BorderFill Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_default_border_fill_when_new() {
        // Arrange & Act
        let bf = BorderFill::new();

        // Assert
        assert_eq!(bf.properties, 0);
        assert!(!bf.has_3d_effect());
        assert!(!bf.has_shadow());
    }

    #[test]
    fn test_should_create_border_fill_with_border_when_with_border() {
        // Arrange & Act
        let bf = BorderFill::with_border();

        // Assert
        assert!(bf.left.is_visible());
        assert!(bf.right.is_visible());
        assert!(bf.top.is_visible());
        assert!(bf.bottom.is_visible());
        assert!(!bf.diagonal.is_visible());
    }

    #[test]
    fn test_should_detect_3d_effect_when_bit_set() {
        // Arrange
        let bf = BorderFill {
            properties: 0x01,
            ..Default::default()
        };

        // Assert
        assert!(bf.has_3d_effect());
        assert!(!bf.has_shadow());
    }

    #[test]
    fn test_should_detect_shadow_when_bit_set() {
        // Arrange
        let bf = BorderFill {
            properties: 0x02,
            ..Default::default()
        };

        // Assert
        assert!(!bf.has_3d_effect());
        assert!(bf.has_shadow());
    }

    #[test]
    fn test_should_return_diagonal_direction_when_extracted() {
        // Arrange - Bits 2-3 = 0b11 (교차)
        let bf = BorderFill {
            properties: 0b1100,
            ..Default::default()
        };

        // Assert
        assert_eq!(bf.diagonal_direction(), 3);
    }

    // ═══════════════════════════════════════════════════════════════
    // BorderLine Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_none_border_line_when_none() {
        // Arrange & Act
        let line = BorderLine::none();

        // Assert
        assert_eq!(line.line_type, BorderLineType::None);
        assert!(!line.is_visible());
    }

    #[test]
    fn test_should_create_solid_border_line_when_solid() {
        // Arrange & Act
        let line = BorderLine::solid(0xFF0000); // Red

        // Assert
        assert_eq!(line.line_type, BorderLineType::Solid);
        assert_eq!(line.thickness, 1);
        assert_eq!(line.color, 0xFF0000);
        assert!(line.is_visible());
    }

    #[test]
    fn test_should_create_thick_border_line_when_thick() {
        // Arrange & Act
        let line = BorderLine::thick(0x0000FF, 5); // Blue, 0.5mm

        // Assert
        assert_eq!(line.thickness, 5);
        assert!(line.is_visible());
    }

    // ═══════════════════════════════════════════════════════════════
    // BorderLineType Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_convert_from_u8_when_valid() {
        assert_eq!(BorderLineType::from_u8(0), BorderLineType::None);
        assert_eq!(BorderLineType::from_u8(1), BorderLineType::Solid);
        assert_eq!(BorderLineType::from_u8(8), BorderLineType::Double);
        assert_eq!(BorderLineType::from_u8(12), BorderLineType::Wave);
    }

    #[test]
    fn test_should_return_none_when_invalid_u8() {
        assert_eq!(BorderLineType::from_u8(100), BorderLineType::None);
        assert_eq!(BorderLineType::from_u8(255), BorderLineType::None);
    }

    // ═══════════════════════════════════════════════════════════════
    // FillInfo Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_none_fill_when_none() {
        // Arrange & Act
        let fill = FillInfo::none();

        // Assert
        assert_eq!(fill.fill_type, 0);
        assert!(!fill.is_solid());
        assert!(!fill.is_gradient());
        assert!(!fill.is_image());
    }

    #[test]
    fn test_should_create_solid_fill_when_solid() {
        // Arrange & Act
        let fill = FillInfo::solid(0xFFFF00); // Yellow

        // Assert
        assert!(fill.is_solid());
        assert_eq!(fill.background_color, 0xFFFF00);
    }

    #[test]
    fn test_should_detect_gradient_when_bit_set() {
        // Arrange
        let fill = FillInfo {
            fill_type: 0x02,
            ..Default::default()
        };

        // Assert
        assert!(!fill.is_solid());
        assert!(fill.is_gradient());
        assert!(!fill.is_image());
    }

    #[test]
    fn test_should_detect_image_when_bit_set() {
        // Arrange
        let fill = FillInfo {
            fill_type: 0x04,
            ..Default::default()
        };

        // Assert
        assert!(fill.is_image());
    }

    // ═══════════════════════════════════════════════════════════════
    // Serialization Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_roundtrip_border_fill_when_serialized() {
        // Arrange
        let bf = BorderFill::with_border();

        // Act
        let json = serde_json::to_string(&bf).expect("Failed to serialize");
        let deserialized: BorderFill = serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(bf.properties, deserialized.properties);
        assert_eq!(bf.left.color, deserialized.left.color);
        assert_eq!(bf.fill.fill_type, deserialized.fill.fill_type);
    }

    #[test]
    fn test_should_roundtrip_fill_gradient_when_serialized() {
        // Arrange
        let gradient = FillGradient {
            gradient_type: GradientType::Radial,
            start_color: 0xFF0000,
            end_color: 0x0000FF,
            angle: 45,
            center_x: 50,
            center_y: 50,
            blur: 10,
        };

        // Act
        let json = serde_json::to_string(&gradient).expect("Failed to serialize");
        let deserialized: FillGradient =
            serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(gradient.gradient_type, deserialized.gradient_type);
        assert_eq!(gradient.start_color, deserialized.start_color);
        assert_eq!(gradient.angle, deserialized.angle);
    }
}
