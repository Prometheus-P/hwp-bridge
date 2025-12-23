// crates/hwp-core/src/parser/docinfo/bin_data.rs

//! BinData (바이너리 데이터) 파서
//!
//! HWP DocInfo 스트림의 BinData 레코드를 파싱합니다.
//! 레코드 태그: 0x12 (BIN_DATA)

use hwp_types::{BinData, BinDataType};
use nom::{IResult, number::complete::le_u16};

use crate::parser::primitives::parse_utf16le_string;

/// BinData 레코드 최소 크기 (바이트)
/// properties(2) = 2
pub const BIN_DATA_MIN_SIZE: usize = 2;

/// BinData 레코드 파싱
///
/// # Format (가변 길이)
/// - properties: u16 = 2 bytes
/// - abs_path: UTF-16LE string (length-prefixed, if Link type)
/// - rel_path: UTF-16LE string (length-prefixed, if Link type)
/// - bin_id: u16 = 2 bytes
/// - extension: UTF-16LE string (length-prefixed)
pub fn parse_bin_data(input: &[u8], id: u16) -> IResult<&[u8], BinData> {
    // 속성 플래그
    let (input, properties) = le_u16(input)?;
    let storage_type = BinDataType::from_value(properties & 0x03);

    // 링크 타입이면 경로 정보가 있음
    let (input, abs_path, rel_path) = if storage_type.is_link() {
        let (input, abs) = parse_utf16le_string(input)?;
        let (input, rel) = parse_utf16le_string(input)?;
        (input, abs, rel)
    } else {
        (input, String::new(), String::new())
    };

    // BinData ID (중복 정보지만 레코드에 포함)
    let (input, _bin_id) = if input.len() >= 2 {
        le_u16(input)?
    } else {
        (input, id)
    };

    // 확장자
    let (input, extension) = if input.len() >= 2 {
        parse_utf16le_string(input)?
    } else {
        (input, String::new())
    };

    Ok((
        input,
        BinData {
            id,
            properties,
            storage_type,
            abs_path,
            rel_path,
            extension,
            data: Vec::new(), // 실제 데이터는 BinData 스트림에서 추출
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════════════

    /// Embedding 타입 BinData 데이터 생성
    fn create_embedding_bindata(extension: &str) -> Vec<u8> {
        let mut data = Vec::new();

        // properties: 0x01 (Embedding)
        data.extend_from_slice(&0x0001u16.to_le_bytes());

        // bin_id: 1
        data.extend_from_slice(&0x0001u16.to_le_bytes());

        // extension
        let ext_utf16: Vec<u16> = extension.encode_utf16().collect();
        data.extend_from_slice(&(ext_utf16.len() as u16).to_le_bytes());
        for ch in ext_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        data
    }

    /// Link 타입 BinData 데이터 생성
    fn create_link_bindata(abs_path: &str, rel_path: &str, extension: &str) -> Vec<u8> {
        let mut data = Vec::new();

        // properties: 0x00 (Link)
        data.extend_from_slice(&0x0000u16.to_le_bytes());

        // abs_path
        let abs_utf16: Vec<u16> = abs_path.encode_utf16().collect();
        data.extend_from_slice(&(abs_utf16.len() as u16).to_le_bytes());
        for ch in abs_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        // rel_path
        let rel_utf16: Vec<u16> = rel_path.encode_utf16().collect();
        data.extend_from_slice(&(rel_utf16.len() as u16).to_le_bytes());
        for ch in rel_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        // bin_id: 1
        data.extend_from_slice(&0x0001u16.to_le_bytes());

        // extension
        let ext_utf16: Vec<u16> = extension.encode_utf16().collect();
        data.extend_from_slice(&(ext_utf16.len() as u16).to_le_bytes());
        for ch in ext_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        data
    }

    // ═══════════════════════════════════════════════════════════════
    // BinData Parser Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_embedding_bindata_when_valid() {
        // Arrange
        let data = create_embedding_bindata("png");

        // Act
        let result = parse_bin_data(&data, 1);

        // Assert
        assert!(result.is_ok());
        let (remaining, bin_data) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(bin_data.id, 1);
        assert_eq!(bin_data.storage_type, BinDataType::Embedding);
        assert_eq!(bin_data.extension, "png");
        assert!(bin_data.abs_path.is_empty());
        assert!(bin_data.rel_path.is_empty());
    }

    #[test]
    fn test_should_parse_link_bindata_with_paths() {
        // Arrange
        let data = create_link_bindata("C:\\Images\\photo.jpg", "photo.jpg", "jpg");

        // Act
        let result = parse_bin_data(&data, 2);

        // Assert
        assert!(result.is_ok());
        let (_, bin_data) = result.unwrap();
        assert_eq!(bin_data.id, 2);
        assert_eq!(bin_data.storage_type, BinDataType::Link);
        assert_eq!(bin_data.abs_path, "C:\\Images\\photo.jpg");
        assert_eq!(bin_data.rel_path, "photo.jpg");
        assert_eq!(bin_data.extension, "jpg");
    }

    #[test]
    fn test_should_extract_compression_flag_from_properties() {
        // Arrange: properties = 0x05 (Embedding + Compressed)
        let mut data = Vec::new();
        data.extend_from_slice(&0x0005u16.to_le_bytes());
        data.extend_from_slice(&0x0001u16.to_le_bytes()); // bin_id
        data.extend_from_slice(&0x0000u16.to_le_bytes()); // empty extension

        // Act
        let (_, bin_data) = parse_bin_data(&data, 1).unwrap();

        // Assert
        assert!(bin_data.is_compressed());
        assert_eq!(bin_data.storage_type, BinDataType::Embedding);
    }

    #[test]
    fn test_should_parse_storage_type_bindata() {
        // Arrange: properties = 0x02 (Storage/OLE)
        let mut data = Vec::new();
        data.extend_from_slice(&0x0002u16.to_le_bytes());
        data.extend_from_slice(&0x0001u16.to_le_bytes()); // bin_id
        // extension: "ole"
        let ext = "ole";
        let ext_utf16: Vec<u16> = ext.encode_utf16().collect();
        data.extend_from_slice(&(ext_utf16.len() as u16).to_le_bytes());
        for ch in ext_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        // Act
        let (_, bin_data) = parse_bin_data(&data, 1).unwrap();

        // Assert
        assert_eq!(bin_data.storage_type, BinDataType::Storage);
        assert!(bin_data.is_ole());
    }

    #[test]
    fn test_should_parse_bindata_with_korean_path() {
        // Arrange
        let data = create_link_bindata("C:\\문서\\이미지.png", "이미지.png", "png");

        // Act
        let result = parse_bin_data(&data, 3);

        // Assert
        assert!(result.is_ok());
        let (_, bin_data) = result.unwrap();
        assert_eq!(bin_data.abs_path, "C:\\문서\\이미지.png");
        assert_eq!(bin_data.rel_path, "이미지.png");
    }

    #[test]
    fn test_should_handle_minimal_bindata_gracefully() {
        // Arrange: properties only
        let data = vec![0x01, 0x00]; // Embedding, no other data

        // Act
        let result = parse_bin_data(&data, 1);

        // Assert
        assert!(result.is_ok());
        let (_, bin_data) = result.unwrap();
        assert_eq!(bin_data.storage_type, BinDataType::Embedding);
        assert!(bin_data.extension.is_empty());
    }
}
