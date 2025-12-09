// crates/hwp-types/src/lib.rs

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// HWP 변환 프로젝트 전반에서 사용하는 공용 에러 타입
#[derive(Error, Debug)]
pub enum HwpError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OLE Storage Error: {0}")]
    OleError(String),

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