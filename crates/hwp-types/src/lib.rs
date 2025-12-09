// crates/hwp-types/src/lib.rs

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        Self { major, minor, build, revision }
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
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.build, self.revision)
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

/// 파싱 결과물(HwpDocument)을 표현하는 최상위 구조체
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HwpDocument {
    pub metadata: DocumentMetadata,
    pub content: String, // 임시로 String, 나중엔 Vec<Section> 등 구조화
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