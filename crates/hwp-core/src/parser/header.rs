// crates/hwp-core/src/parser/header.rs

use hwp_types::{DocumentProperties, FileHeader, HWP_SIGNATURE, HwpError, HwpVersion};

/// FileHeader 크기 (256 bytes)
pub const FILE_HEADER_SIZE: usize = 256;

/// FileHeader 바이트 배열에서 파싱
pub fn parse_file_header(data: &[u8]) -> Result<FileHeader, HwpError> {
    if data.len() < FILE_HEADER_SIZE {
        return Err(HwpError::ParseError(format!(
            "FileHeader too short: {} bytes (expected {})",
            data.len(),
            FILE_HEADER_SIZE
        )));
    }

    // 시그니처 검증 (offset 0, 32 bytes)
    let signature = &data[0..32];
    if signature != HWP_SIGNATURE.as_slice() {
        return Err(HwpError::InvalidSignature);
    }

    // 버전 파싱 (offset 32, 4 bytes)
    let version_bytes: [u8; 4] = data[32..36].try_into().unwrap();
    let version = HwpVersion::from_bytes(version_bytes);

    // 속성 파싱 (offset 36, 4 bytes, little-endian)
    let properties_bits = u32::from_le_bytes(data[36..40].try_into().unwrap());
    let properties = DocumentProperties::from_bits(properties_bits);

    Ok(FileHeader {
        version,
        properties,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_header(version: [u8; 4], properties: u32) -> Vec<u8> {
        let mut data = vec![0u8; FILE_HEADER_SIZE];

        // 시그니처
        data[0..32].copy_from_slice(HWP_SIGNATURE);

        // 버전
        data[32..36].copy_from_slice(&version);

        // 속성
        data[36..40].copy_from_slice(&properties.to_le_bytes());

        data
    }

    #[test]
    fn test_should_parse_valid_header() {
        // HWP 5.1.0.0, 압축됨 (bit 0)
        let data = create_test_header([0, 0, 1, 5], 0b0001);
        let header = parse_file_header(&data).unwrap();

        assert_eq!(header.version.major, 5);
        assert_eq!(header.version.minor, 1);
        assert!(header.properties.is_compressed());
        assert!(!header.properties.is_encrypted());
    }

    #[test]
    fn test_should_fail_on_invalid_signature() {
        let mut data = vec![0u8; FILE_HEADER_SIZE];
        data[0..10].copy_from_slice(b"INVALID!!!");

        let result = parse_file_header(&data);
        assert!(matches!(result, Err(HwpError::InvalidSignature)));
    }

    #[test]
    fn test_should_fail_on_short_data() {
        let data = vec![0u8; 100]; // Too short

        let result = parse_file_header(&data);
        assert!(matches!(result, Err(HwpError::ParseError(_))));
    }

    #[test]
    fn test_should_detect_encrypted_document() {
        // 암호화 플래그 (bit 1)
        let data = create_test_header([0, 0, 1, 5], 0b0010);
        let header = parse_file_header(&data).unwrap();

        assert!(header.properties.is_encrypted());
        assert!(matches!(header.validate(), Err(HwpError::Encrypted)));
    }

    #[test]
    fn test_should_detect_distribution_document() {
        // 배포용 문서 플래그 (bit 2)
        let data = create_test_header([0, 0, 1, 5], 0b0100);
        let header = parse_file_header(&data).unwrap();

        assert!(header.properties.is_distribution());
        assert!(matches!(header.validate(), Err(HwpError::DistributionOnly)));
    }

    #[test]
    fn test_should_reject_old_version() {
        // HWP 3.0.0.0
        let data = create_test_header([0, 0, 0, 3], 0b0001);
        let header = parse_file_header(&data).unwrap();

        assert!(!header.version.is_supported());
        assert!(matches!(
            header.validate(),
            Err(HwpError::UnsupportedVersion(_))
        ));
    }

    #[test]
    fn test_should_pass_validation_for_normal_document() {
        // HWP 5.1.0.0, 압축만 됨
        let data = create_test_header([0, 0, 1, 5], 0b0001);
        let header = parse_file_header(&data).unwrap();

        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_should_parse_all_property_flags() {
        // 모든 플래그 설정
        let flags = 0b1111_1111_1111_1111u32;
        let data = create_test_header([0, 0, 1, 5], flags);
        let header = parse_file_header(&data).unwrap();

        assert!(header.properties.is_compressed());
        assert!(header.properties.is_encrypted());
        assert!(header.properties.is_distribution());
        assert!(header.properties.has_script());
        assert!(header.properties.has_drm());
        assert!(header.properties.has_xml_template());
        assert!(header.properties.has_history());
        assert!(header.properties.has_signature());
        assert!(header.properties.has_cert_encryption());
        assert!(header.properties.is_ccl());
        assert!(header.properties.is_mobile_optimized());
        assert!(header.properties.has_track_changes());
        assert!(header.properties.is_kogl());
    }
}
