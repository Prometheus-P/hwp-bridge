// crates/hwp-types/src/face_name.rs

//! 글꼴(FaceName) 타입
//!
//! HWP 문서에서 사용하는 글꼴 정보를 정의합니다.

use serde::{Deserialize, Serialize};

/// 글꼴 정의 (DocInfo 스트림에서 파싱)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FaceName {
    /// 속성 플래그
    /// Bit 0: 대체 글꼴 존재
    /// Bit 1: 글꼴 유형 정보 존재
    /// Bit 2: 기본 글꼴 존재
    /// Bit 7: PANOSE 정보 존재
    pub properties: u8,
    /// 글꼴 이름 (예: "함초롬돋움", "맑은 고딕")
    pub name: String,
    /// 대체 글꼴 종류
    pub substitute_type: SubstituteFontType,
    /// 대체 글꼴 이름
    pub substitute_name: String,
    /// PANOSE 글꼴 분류 정보 (10 bytes)
    pub panose: Option<Panose>,
    /// 기본 글꼴 이름 (글꼴을 찾을 수 없을 때 사용)
    pub default_name: String,
}

impl FaceName {
    /// 새 FaceName 생성
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            substitute_name: name.clone(),
            default_name: name,
            ..Default::default()
        }
    }

    /// 한글 기본 글꼴 (함초롬돋움)
    pub fn korean_default() -> Self {
        Self::new("함초롬돋움")
    }

    /// 영문 기본 글꼴 (함초롬돋움)
    pub fn english_default() -> Self {
        Self::new("함초롬돋움")
    }

    /// 대체 글꼴 존재 여부
    pub fn has_substitute(&self) -> bool {
        self.properties & 0x01 != 0
    }

    /// 글꼴 유형 정보 존재 여부
    pub fn has_font_type_info(&self) -> bool {
        self.properties & 0x02 != 0
    }

    /// 기본 글꼴 존재 여부
    pub fn has_default(&self) -> bool {
        self.properties & 0x04 != 0
    }

    /// PANOSE 정보 존재 여부
    pub fn has_panose(&self) -> bool {
        self.properties & 0x80 != 0
    }

    /// 속성 플래그 설정 (빌더 패턴)
    pub fn with_properties(mut self, properties: u8) -> Self {
        self.properties = properties;
        self
    }

    /// 대체 글꼴 설정
    pub fn with_substitute(
        mut self,
        sub_type: SubstituteFontType,
        sub_name: impl Into<String>,
    ) -> Self {
        self.properties |= 0x01;
        self.substitute_type = sub_type;
        self.substitute_name = sub_name.into();
        self
    }

    /// PANOSE 정보 설정
    pub fn with_panose(mut self, panose: Panose) -> Self {
        self.properties |= 0x80;
        self.panose = Some(panose);
        self
    }

    /// 기본 글꼴 설정
    pub fn with_default_name(mut self, default_name: impl Into<String>) -> Self {
        self.properties |= 0x04;
        self.default_name = default_name.into();
        self
    }
}

/// 대체 글꼴 종류
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SubstituteFontType {
    /// 알 수 없음
    #[default]
    Unknown = 0,
    /// TrueType
    TrueType = 1,
    /// Type1
    Type1 = 2,
}

impl SubstituteFontType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::TrueType,
            2 => Self::Type1,
            _ => Self::Unknown,
        }
    }
}

/// PANOSE 글꼴 분류 정보
///
/// PANOSE는 글꼴의 시각적 특성을 10개의 숫자로 분류하는 표준입니다.
/// 이 정보를 사용하여 유사한 글꼴을 대체할 수 있습니다.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Panose {
    /// 글꼴 패밀리 종류 (0-5)
    /// 0=Any, 1=No Fit, 2=Latin Text, 3=Latin Hand Written,
    /// 4=Latin Decorative, 5=Latin Symbol
    pub family_type: u8,
    /// 세리프 스타일 (0-15)
    pub serif_style: u8,
    /// 굵기 (1-11, 5=Normal, 8=Bold)
    pub weight: u8,
    /// 비율 (1-9)
    pub proportion: u8,
    /// 대비 (1-9)
    pub contrast: u8,
    /// 획 변화 (1-9)
    pub stroke_variation: u8,
    /// 팔/다리 스타일 (1-11)
    pub arm_style: u8,
    /// 글자 형태 (1-15)
    pub letterform: u8,
    /// 중간선 (1-13)
    pub midline: u8,
    /// X-높이 (1-7)
    pub x_height: u8,
}

impl Panose {
    /// 새 PANOSE 정보 생성
    pub fn new(bytes: [u8; 10]) -> Self {
        Self {
            family_type: bytes[0],
            serif_style: bytes[1],
            weight: bytes[2],
            proportion: bytes[3],
            contrast: bytes[4],
            stroke_variation: bytes[5],
            arm_style: bytes[6],
            letterform: bytes[7],
            midline: bytes[8],
            x_height: bytes[9],
        }
    }

    /// 바이트 배열로 변환
    pub fn to_bytes(&self) -> [u8; 10] {
        [
            self.family_type,
            self.serif_style,
            self.weight,
            self.proportion,
            self.contrast,
            self.stroke_variation,
            self.arm_style,
            self.letterform,
            self.midline,
            self.x_height,
        ]
    }

    /// Latin Text 기본값 (본문용 글꼴)
    pub fn latin_text_default() -> Self {
        Self {
            family_type: 2, // Latin Text
            serif_style: 0,
            weight: 5,     // Normal
            proportion: 3, // Modern
            contrast: 0,
            stroke_variation: 0,
            arm_style: 0,
            letterform: 0,
            midline: 0,
            x_height: 0,
        }
    }

    /// 글꼴이 굵은지 확인 (weight >= 7)
    pub fn is_bold(&self) -> bool {
        self.weight >= 7
    }

    /// 글꼴이 세리프인지 확인
    pub fn is_serif(&self) -> bool {
        // serif_style 2-10은 세리프 스타일
        self.serif_style >= 2 && self.serif_style <= 10
    }
}

/// 글꼴 언어 종류 (HWP는 7개 언어별로 글꼴 지정)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FontLanguage {
    /// 한글
    Korean = 0,
    /// 영문
    English = 1,
    /// 한자
    Hanja = 2,
    /// 일본어
    Japanese = 3,
    /// 기타
    Other = 4,
    /// 기호
    Symbol = 5,
    /// 사용자 정의
    User = 6,
}

impl FontLanguage {
    /// 전체 언어 목록 반환
    pub fn all() -> [Self; 7] {
        [
            Self::Korean,
            Self::English,
            Self::Hanja,
            Self::Japanese,
            Self::Other,
            Self::Symbol,
            Self::User,
        ]
    }

    /// 인덱스에서 변환
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Korean),
            1 => Some(Self::English),
            2 => Some(Self::Hanja),
            3 => Some(Self::Japanese),
            4 => Some(Self::Other),
            5 => Some(Self::Symbol),
            6 => Some(Self::User),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // FaceName Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_face_name_when_new() {
        // Arrange & Act
        let face = FaceName::new("맑은 고딕");

        // Assert
        assert_eq!(face.name, "맑은 고딕");
        assert_eq!(face.substitute_name, "맑은 고딕");
        assert_eq!(face.default_name, "맑은 고딕");
    }

    #[test]
    fn test_should_create_korean_default_when_korean_default() {
        // Arrange & Act
        let face = FaceName::korean_default();

        // Assert
        assert_eq!(face.name, "함초롬돋움");
    }

    #[test]
    fn test_should_detect_substitute_when_bit_set() {
        // Arrange
        let face = FaceName {
            properties: 0x01,
            ..Default::default()
        };

        // Assert
        assert!(face.has_substitute());
        assert!(!face.has_font_type_info());
        assert!(!face.has_default());
        assert!(!face.has_panose());
    }

    #[test]
    fn test_should_detect_panose_when_bit_set() {
        // Arrange
        let face = FaceName {
            properties: 0x80,
            ..Default::default()
        };

        // Assert
        assert!(face.has_panose());
    }

    #[test]
    fn test_should_chain_builder_methods() {
        // Arrange & Act
        let face = FaceName::new("나눔고딕")
            .with_substitute(SubstituteFontType::TrueType, "Arial")
            .with_default_name("굴림");

        // Assert
        assert!(face.has_substitute());
        assert!(face.has_default());
        assert_eq!(face.substitute_type, SubstituteFontType::TrueType);
        assert_eq!(face.substitute_name, "Arial");
        assert_eq!(face.default_name, "굴림");
    }

    #[test]
    fn test_should_set_panose_when_with_panose() {
        // Arrange
        let panose = Panose::latin_text_default();

        // Act
        let face = FaceName::new("Arial").with_panose(panose);

        // Assert
        assert!(face.has_panose());
        assert!(face.panose.is_some());
    }

    // ═══════════════════════════════════════════════════════════════
    // SubstituteFontType Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_convert_substitute_type_from_u8() {
        assert_eq!(SubstituteFontType::from_u8(0), SubstituteFontType::Unknown);
        assert_eq!(SubstituteFontType::from_u8(1), SubstituteFontType::TrueType);
        assert_eq!(SubstituteFontType::from_u8(2), SubstituteFontType::Type1);
        assert_eq!(SubstituteFontType::from_u8(99), SubstituteFontType::Unknown);
    }

    // ═══════════════════════════════════════════════════════════════
    // Panose Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_panose_from_bytes() {
        // Arrange
        let bytes: [u8; 10] = [2, 1, 6, 3, 5, 4, 3, 2, 1, 4];

        // Act
        let panose = Panose::new(bytes);

        // Assert
        assert_eq!(panose.family_type, 2);
        assert_eq!(panose.serif_style, 1);
        assert_eq!(panose.weight, 6);
    }

    #[test]
    fn test_should_convert_panose_to_bytes() {
        // Arrange
        let panose = Panose {
            family_type: 2,
            serif_style: 1,
            weight: 8, // Bold
            proportion: 3,
            contrast: 5,
            stroke_variation: 4,
            arm_style: 3,
            letterform: 2,
            midline: 1,
            x_height: 4,
        };

        // Act
        let bytes = panose.to_bytes();

        // Assert
        assert_eq!(bytes[0], 2);
        assert_eq!(bytes[2], 8);
    }

    #[test]
    fn test_should_detect_bold_when_weight_high() {
        // Arrange
        let panose = Panose {
            weight: 8, // Bold
            ..Default::default()
        };

        // Assert
        assert!(panose.is_bold());
    }

    #[test]
    fn test_should_not_detect_bold_when_weight_normal() {
        // Arrange
        let panose = Panose {
            weight: 5, // Normal
            ..Default::default()
        };

        // Assert
        assert!(!panose.is_bold());
    }

    #[test]
    fn test_should_detect_serif_when_serif_style_in_range() {
        // Arrange
        let panose = Panose {
            serif_style: 5,
            ..Default::default()
        };

        // Assert
        assert!(panose.is_serif());
    }

    #[test]
    fn test_should_not_detect_serif_when_sans_serif() {
        // Arrange
        let panose = Panose {
            serif_style: 11, // Sans Serif
            ..Default::default()
        };

        // Assert
        assert!(!panose.is_serif());
    }

    // ═══════════════════════════════════════════════════════════════
    // FontLanguage Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_return_all_languages() {
        let all = FontLanguage::all();
        assert_eq!(all.len(), 7);
        assert_eq!(all[0], FontLanguage::Korean);
        assert_eq!(all[6], FontLanguage::User);
    }

    #[test]
    fn test_should_convert_from_index() {
        assert_eq!(FontLanguage::from_index(0), Some(FontLanguage::Korean));
        assert_eq!(FontLanguage::from_index(1), Some(FontLanguage::English));
        assert_eq!(FontLanguage::from_index(7), None);
    }

    // ═══════════════════════════════════════════════════════════════
    // Serialization Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_roundtrip_face_name_when_serialized() {
        // Arrange
        let face = FaceName::new("나눔고딕")
            .with_substitute(SubstituteFontType::TrueType, "Arial")
            .with_panose(Panose::latin_text_default());

        // Act
        let json = serde_json::to_string(&face).expect("Failed to serialize");
        let deserialized: FaceName = serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(face.name, deserialized.name);
        assert_eq!(face.substitute_type, deserialized.substitute_type);
        assert!(deserialized.panose.is_some());
    }

    #[test]
    fn test_should_roundtrip_panose_when_serialized() {
        // Arrange
        let panose = Panose::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Act
        let json = serde_json::to_string(&panose).expect("Failed to serialize");
        let deserialized: Panose = serde_json::from_str(&json).expect("Failed to deserialize");

        // Assert
        assert_eq!(panose.to_bytes(), deserialized.to_bytes());
    }
}
