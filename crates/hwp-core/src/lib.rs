// crates/hwp-core/src/lib.rs

//! HWP 파일 파싱 및 변환 핵심 엔진
//!
//! HWP 5.0 포맷의 OLE 컨테이너를 열고, FileHeader를 파싱하여
//! 암호화/배포용 문서를 Fail-Fast로 걸러냅니다.

pub mod parser;

pub use parser::{parse_file_header, HwpOleFile, HwpTextExtractor};

// Re-export common types from hwp-types
pub use hwp_types::{
    DocumentProperties, FileHeader, HwpDocument, HwpError, HwpVersion, HWP_SIGNATURE,
};
