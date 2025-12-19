// crates/hwp-core/src/parser/section.rs

//! HWP BodyText Section 파싱
//!
//! BodyText/Section 스트림은 zlib으로 압축되어 있습니다.
//! 압축 해제 후 레코드 단위로 파싱합니다.

use flate2::read::ZlibDecoder;
use hwp_types::HwpError;
use std::io::Read;

use super::record::{Record, RecordIterator};

/// Default max decompressed bytes per section (safety)
pub const DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION: usize = 64 * 1024 * 1024; // 64MB
/// Default max records per section (safety)
pub const DEFAULT_MAX_RECORDS_PER_SECTION: usize = 200_000;

#[derive(Debug, Clone, Copy)]
pub struct SectionLimits {
    pub max_decompressed_bytes: usize,
    pub max_records: usize,
}

impl Default for SectionLimits {
    fn default() -> Self {
        Self {
            max_decompressed_bytes: DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION,
            max_records: DEFAULT_MAX_RECORDS_PER_SECTION,
        }
    }
}

/// Section 압축 해제
///
/// HWP의 BodyText/Section은 zlib (deflate) 압축입니다.
/// 압축 해제 후 레코드 스트림을 반환합니다.
pub fn decompress_section(data: &[u8]) -> Result<Vec<u8>, HwpError> {
    decompress_section_with_limits(data, DEFAULT_MAX_DECOMPRESSED_BYTES_PER_SECTION)
}

/// Section 압축 해제 (상한 적용)
pub fn decompress_section_with_limits(
    data: &[u8],
    max_decompressed_bytes: usize,
) -> Result<Vec<u8>, HwpError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    let mut total: usize = 0;

    loop {
        let n = decoder
            .read(&mut buf)
            .map_err(|e| HwpError::ParseError(format!("Decompression failed: {}", e)))?;
        if n == 0 {
            break;
        }
        if total + n > max_decompressed_bytes {
            return Err(HwpError::SizeLimitExceeded(format!(
                "Decompressed section exceeds limit: {} > {} bytes",
                total + n,
                max_decompressed_bytes
            )));
        }
        out.extend_from_slice(&buf[..n]);
        total += n;
    }

    Ok(out)
}

/// Section에서 레코드 파싱
/// Section 레코드 파싱 (기본: compressed, default limits)
pub fn parse_section_records(compressed_data: &[u8]) -> Result<Vec<Record>, HwpError> {
    parse_section_records_with_options(compressed_data, true, SectionLimits::default())
}

/// Section 레코드 파싱 (옵션)
pub fn parse_section_records_with_options(
    data: &[u8],
    is_compressed: bool,
    limits: SectionLimits,
) -> Result<Vec<Record>, HwpError> {
    let decompressed = if is_compressed {
        decompress_section_with_limits(data, limits.max_decompressed_bytes)?
    } else {
        if data.len() > limits.max_decompressed_bytes {
            return Err(HwpError::SizeLimitExceeded(format!(
                "Uncompressed section exceeds limit: {} > {} bytes",
                data.len(),
                limits.max_decompressed_bytes
            )));
        }
        data.to_vec()
    };

    let mut records = Vec::new();
    for result in RecordIterator::new(&decompressed) {
        records.push(result?);
        if records.len() > limits.max_records {
            return Err(HwpError::SizeLimitExceeded(format!(
                "Section record count exceeds limit: {} > {}",
                records.len(),
                limits.max_records
            )));
        }
    }

    Ok(records)
}

/// PARA_TEXT 레코드에서 텍스트 추출
///
/// PARA_TEXT (0x43) 레코드의 데이터는 UTF-16LE로 인코딩된 텍스트입니다.
/// 특수 제어 문자를 처리합니다.
pub fn extract_text_from_para_text(data: &[u8]) -> Result<String, HwpError> {
    if data.is_empty() {
        return Ok(String::new());
    }

    // UTF-16LE는 2바이트 단위
    if !data.len().is_multiple_of(2) {
        return Err(HwpError::ParseError(
            "Invalid UTF-16LE data: odd number of bytes".into(),
        ));
    }

    // 바이트를 u16 배열로 변환
    let chars: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // HWP 제어 문자 필터링
    // 0x00-0x1F: HWP 내부 제어 문자 (대부분 건너뜀)
    // 예외: 0x0A (줄바꿈), 0x0D (캐리지 리턴), 0x09 (탭)
    let filtered: Vec<u16> = chars
        .into_iter()
        .filter(|&c| {
            // 일반 문자 (0x20 이상) 또는 허용된 제어 문자
            c >= 0x20 || c == 0x0A || c == 0x0D || c == 0x09
        })
        .collect();

    // UTF-16 디코딩
    String::from_utf16(&filtered)
        .map_err(|e| HwpError::ParseError(format!("UTF-16 decode error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;

    // ═══════════════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════════════

    /// 테스트 데이터를 zlib 압축
    fn compress_data(data: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    /// 레코드 헤더 생성 (normal size)
    fn create_record_header(tag_id: u16, level: u16, size: u32) -> Vec<u8> {
        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
        dword.to_le_bytes().to_vec()
    }

    /// UTF-16LE로 인코딩된 문자열 생성
    fn encode_utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
    }

    // ═══════════════════════════════════════════════════════════════
    // Decompression Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_decompress_zlib_data() {
        // Arrange
        let original = b"Hello, HWP World!";
        let compressed = compress_data(original);

        // Act
        let result = decompress_section(&compressed);

        // Assert
        assert!(result.is_ok(), "Should decompress successfully");
        assert_eq!(result.unwrap(), original.to_vec());
    }

    #[test]
    fn test_should_return_error_when_invalid_zlib() {
        // Arrange: Invalid zlib data
        let invalid = vec![0x00, 0x01, 0x02, 0x03];

        // Act
        let result = decompress_section(&invalid);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_should_decompress_empty_data() {
        // Arrange: zlib compressed empty data
        let compressed = compress_data(&[]);

        // Act
        let result = decompress_section(&compressed);

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // ═══════════════════════════════════════════════════════════════
    // Section Parsing Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_section_with_records() {
        // Arrange: Create a section with one PARA_TEXT record
        let text_data = encode_utf16le("안녕");
        let mut section_data = create_record_header(0x43, 0, text_data.len() as u32);
        section_data.extend_from_slice(&text_data);

        let compressed = compress_data(&section_data);

        // Act
        let result = parse_section_records(&compressed);

        // Assert
        assert!(result.is_ok(), "Should parse section records");
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].header.tag_id, 0x43); // PARA_TEXT
    }

    // ═══════════════════════════════════════════════════════════════
    // Text Extraction Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_extract_korean_text() {
        // Arrange: UTF-16LE encoded Korean text
        let data = encode_utf16le("안녕하세요");

        // Act
        let result = extract_text_from_para_text(&data);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "안녕하세요");
    }

    #[test]
    fn test_should_extract_mixed_text() {
        // Arrange: Mixed Korean and English
        let data = encode_utf16le("Hello 세계!");

        // Act
        let result = extract_text_from_para_text(&data);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello 세계!");
    }

    #[test]
    fn test_should_handle_empty_text() {
        // Arrange: Empty
        let data: Vec<u8> = vec![];

        // Act
        let result = extract_text_from_para_text(&data);

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_should_skip_control_characters() {
        // Arrange: Text with HWP control characters (0x00-0x1F are special in HWP)
        // 0x02 = section/column break, 0x03 = field start
        let mut data = vec![];
        // 'A' (0x0041)
        data.extend_from_slice(&0x0041u16.to_le_bytes());
        // Control char 0x02 (섹션 나눔)
        data.extend_from_slice(&0x0002u16.to_le_bytes());
        // 'B' (0x0042)
        data.extend_from_slice(&0x0042u16.to_le_bytes());

        // Act
        let result = extract_text_from_para_text(&data);

        // Assert
        assert!(result.is_ok());
        // Control characters should be filtered or replaced
        let text = result.unwrap();
        assert!(text.contains('A'));
        assert!(text.contains('B'));
    }
}
