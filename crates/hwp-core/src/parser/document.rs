// crates/hwp-core/src/parser/document.rs

//! 문서 수준 텍스트 추출
//!
//! HWP 파일에서 모든 텍스트를 추출하는 고수준 API를 제공합니다.

use std::io::{Read, Seek};

use hwp_types::HwpError;

use super::ole::HwpOleFile;
use super::record::tags;
use super::section::{
    SectionLimits, extract_text_from_para_text, parse_section_records_with_options,
};

/// HWP 문서에서 텍스트를 추출하는 구조체
pub struct HwpTextExtractor<F: Read + Seek> {
    ole: HwpOleFile<F>,
    limits: SectionLimits,
}

impl<F: Read + Seek> HwpTextExtractor<F> {
    /// HWP 파일을 열고 TextExtractor 생성
    pub fn open(inner: F) -> Result<Self, HwpError> {
        let ole = HwpOleFile::open(inner)?;
        Ok(Self {
            ole,
            limits: SectionLimits::default(),
        })
    }

    /// 리소스 상한 설정
    pub fn with_limits(mut self, limits: SectionLimits) -> Self {
        self.limits = limits;
        self
    }

    /// 모든 섹션에서 텍스트 추출
    ///
    /// 각 섹션의 PARA_TEXT 레코드에서 텍스트를 추출하여
    /// 섹션 구분자(----)와 함께 연결합니다.
    pub fn extract_all_text(&mut self) -> Result<String, HwpError> {
        let mut all_text = Vec::new();
        let mut section_idx = 0;

        loop {
            if section_idx >= self.limits.max_sections {
                return Err(HwpError::SizeLimitExceeded(format!(
                    "Section count exceeds limit: {} > {}",
                    section_idx, self.limits.max_sections
                )));
            }
            match self.ole.read_section(section_idx) {
                Ok(compressed_data) => {
                    let section_text = self.extract_section_text(&compressed_data)?;
                    if !section_text.is_empty() {
                        all_text.push(section_text);
                    }
                    section_idx += 1;
                }
                Err(HwpError::NotFound(_)) => break, // 더 이상 섹션 없음
                Err(e) => return Err(e),
            }
        }

        // 섹션들을 줄바꿈으로 연결
        Ok(all_text.join("\n\n"))
    }

    /// 단일 섹션에서 텍스트 추출
    fn extract_section_text(&self, compressed_data: &[u8]) -> Result<String, HwpError> {
        let is_compressed = self.ole.header().properties.is_compressed();
        let records =
            parse_section_records_with_options(compressed_data, is_compressed, self.limits)?;
        let mut paragraphs = Vec::new();

        for record in records {
            if record.header.tag_id == tags::PARA_TEXT {
                let text = extract_text_from_para_text(&record.data)?;
                if !text.is_empty() {
                    paragraphs.push(text);
                }
            }
        }

        Ok(paragraphs.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::write::DeflateEncoder;
    use std::io::{Cursor, Write};

    // ═══════════════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════════════

    /// 테스트용 최소 HWP 파일 생성
    fn create_minimal_hwp_with_text(text: &str) -> Vec<u8> {
        use cfb::CompoundFile;

        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut cfb = CompoundFile::create(cursor).expect("Failed to create CFB");

            // FileHeader 스트림 생성
            let file_header = create_file_header();
            let mut stream = cfb
                .create_stream("/FileHeader")
                .expect("Failed to create FileHeader");
            stream
                .write_all(&file_header)
                .expect("Failed to write FileHeader");

            // BodyText 스토리지 생성 (먼저 부모 디렉터리 생성)
            cfb.create_storage("/BodyText")
                .expect("Failed to create BodyText storage");

            // Section0 스트림 생성 (압축된 레코드)
            let section_data = create_section_with_text(text);
            let compressed = compress_data(&section_data);
            let mut stream = cfb
                .create_stream("/BodyText/Section0")
                .expect("Failed to create Section0");
            stream
                .write_all(&compressed)
                .expect("Failed to write Section0");

            cfb.flush().expect("Failed to flush CFB");
        }
        buffer
    }

    fn create_minimal_hwp_with_sections(texts: &[&str]) -> Vec<u8> {
        use cfb::CompoundFile;

        let mut buffer = Vec::new();
        {
            let cursor = Cursor::new(&mut buffer);
            let mut cfb = CompoundFile::create(cursor).expect("Failed to create CFB");

            let file_header = create_file_header();
            let mut stream = cfb
                .create_stream("/FileHeader")
                .expect("Failed to create FileHeader");
            stream
                .write_all(&file_header)
                .expect("Failed to write FileHeader");

            cfb.create_storage("/BodyText")
                .expect("Failed to create BodyText storage");

            for (idx, text) in texts.iter().enumerate() {
                let section_data = create_section_with_text(text);
                let compressed = compress_data(&section_data);
                let mut stream = cfb
                    .create_stream(format!("/BodyText/Section{}", idx))
                    .expect("Failed to create Section stream");
                stream
                    .write_all(&compressed)
                    .expect("Failed to write Section stream");
            }

            cfb.flush().expect("Failed to flush CFB");
        }
        buffer
    }

    /// 테스트용 FileHeader (256 bytes)
    fn create_file_header() -> Vec<u8> {
        let mut header = vec![0u8; 256];

        // Signature
        let signature = b"HWP Document File\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        header[0..32].copy_from_slice(signature);

        // Version: 5.1.0.0 (HWP stores as [revision, build, minor, major])
        // from_bytes reads: major=bytes[3], minor=bytes[2], build=bytes[1], revision=bytes[0]
        header[32] = 0; // revision (bytes[0])
        header[33] = 0; // build (bytes[1])
        header[34] = 1; // minor (bytes[2])
        header[35] = 5; // major (bytes[3])

        // Properties: compressed, not encrypted, not distribution
        let props: u32 = 0x01; // compressed only
        header[36..40].copy_from_slice(&props.to_le_bytes());

        header
    }

    /// PARA_TEXT 레코드가 포함된 섹션 데이터 생성
    fn create_section_with_text(text: &str) -> Vec<u8> {
        let text_bytes = encode_utf16le(text);
        let mut data = create_record_header(0x43, 0, text_bytes.len() as u32);
        data.extend_from_slice(&text_bytes);
        data
    }

    /// 레코드 헤더 생성
    fn create_record_header(tag_id: u16, level: u16, size: u32) -> Vec<u8> {
        let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
        dword.to_le_bytes().to_vec()
    }

    /// UTF-16LE 인코딩
    fn encode_utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
    }

    /// zlib 압축
    fn compress_data(data: &[u8]) -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(9));
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    // ═══════════════════════════════════════════════════════════════
    // HwpTextExtractor Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_extract_korean_text_from_hwp() {
        // Arrange
        let hwp_data = create_minimal_hwp_with_text("안녕하세요");
        let cursor = Cursor::new(hwp_data);

        // Act
        let mut extractor = HwpTextExtractor::open(cursor).expect("Failed to open HWP");
        let result = extractor.extract_all_text();

        // Assert
        assert!(result.is_ok(), "Should extract text: {:?}", result.err());
        let text = result.unwrap();
        assert!(
            text.contains("안녕하세요"),
            "Should contain Korean text, got: {}",
            text
        );
    }

    #[test]
    fn test_should_extract_english_text_from_hwp() {
        // Arrange
        let hwp_data = create_minimal_hwp_with_text("Hello World");
        let cursor = Cursor::new(hwp_data);

        // Act
        let mut extractor = HwpTextExtractor::open(cursor).expect("Failed to open HWP");
        let result = extractor.extract_all_text();

        // Assert
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_should_extract_mixed_text_from_hwp() {
        // Arrange
        let hwp_data = create_minimal_hwp_with_text("Hello 세계!");
        let cursor = Cursor::new(hwp_data);

        // Act
        let mut extractor = HwpTextExtractor::open(cursor).expect("Failed to open HWP");
        let result = extractor.extract_all_text();

        // Assert
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("세계"));
    }

    #[test]
    fn test_should_handle_empty_section() {
        // Arrange: HWP with empty text
        let hwp_data = create_minimal_hwp_with_text("");
        let cursor = Cursor::new(hwp_data);

        // Act
        let mut extractor = HwpTextExtractor::open(cursor).expect("Failed to open HWP");
        let result = extractor.extract_all_text();

        // Assert
        assert!(result.is_ok());
        // Empty text is valid
    }

    #[test]
    fn test_should_enforce_max_sections_limit() {
        // Arrange
        let hwp_data = create_minimal_hwp_with_sections(&["One", "Two"]);
        let cursor = Cursor::new(hwp_data);
        let limits = SectionLimits {
            max_sections: 1,
            ..SectionLimits::default()
        };

        // Act
        let mut extractor = HwpTextExtractor::open(cursor)
            .expect("Failed to open HWP")
            .with_limits(limits);
        let result = extractor.extract_all_text();

        // Assert
        assert!(
            matches!(result, Err(HwpError::SizeLimitExceeded(_))),
            "Expected SizeLimitExceeded, got: {:?}",
            result
        );
    }

    #[test]
    fn test_should_fail_for_invalid_hwp() {
        // Arrange: Random bytes, not a valid HWP
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03];
        let cursor = Cursor::new(invalid_data);

        // Act
        let result = HwpTextExtractor::open(cursor);

        // Assert
        assert!(result.is_err(), "Should fail for invalid HWP file");
    }
}
